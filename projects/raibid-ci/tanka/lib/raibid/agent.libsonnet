// agent.libsonnet - raibid-agent ScaledJob configuration
// KEDA ScaledJob for auto-scaling build agents based on queue depth

local k = import '../k.libsonnet';
local config = import './config.libsonnet';
local util = import './util.libsonnet';
local keda = import '../charts/keda.libsonnet';

{
  // Create raibid-agent ScaledJob with KEDA autoscaling
  // Parameters:
  //   namespace: Kubernetes namespace
  //   name: ScaledJob name (default: 'raibid-agent')
  //   image: Container image
  //   redisAddress: Redis server address (default: 'redis-master:6379')
  //   streamName: Redis stream name (default: 'raibid:jobs')
  //   consumerGroup: Consumer group name (default: 'raibid-agents')
  //   pollingInterval: Polling interval in seconds (default: 10)
  //   maxReplicaCount: Maximum number of agents (default: 10)
  //   lagThreshold: Queue lag threshold to trigger scaling (default: 5)
  //   env: Additional environment variables
  new(
    namespace,
    name='raibid-agent',
    image='raibid-agent:latest',
    redisAddress='redis-master:6379',
    streamName='raibid:jobs',
    consumerGroup='raibid-agents',
    pollingInterval=10,
    maxReplicaCount=10,
    lagThreshold=5,
    env={}
  ):: {
    local labels = config.labels.forComponent('agent'),

    // Job template for the agent
    local jobTemplate = {
      metadata: {
        labels: labels,
      },
      spec: {
        template: {
          metadata: {
            labels: labels,
          },
          spec: {
            // Don't restart on failure - let KEDA handle scaling
            restartPolicy: 'Never',

            // Security context
            securityContext: {
              runAsNonRoot: true,
              runAsUser: 1000,
              fsGroup: 1000,
            },

            containers: [
              {
                name: 'agent',
                image: image,
                imagePullPolicy: 'IfNotPresent',

                // Environment variables
                env: [
                  util.env.value('RUST_LOG', 'info'),
                  util.env.value('REDIS_URL', 'redis://%s' % redisAddress),
                  util.env.value('STREAM_NAME', streamName),
                  util.env.value('CONSUMER_GROUP', consumerGroup),
                  util.env.configMap('QUEUE_NAME', 'raibid-config', 'QUEUE_NAME'),
                ] + [
                  util.env.value(k, env[k])
                  for k in std.objectFields(env)
                ],

                // Resource limits - agents need more resources for builds
                resources: {
                  requests: {
                    cpu: '1000m',
                    memory: '2Gi',
                  },
                  limits: {
                    cpu: '2000m',
                    memory: '4Gi',
                  },
                },

                // Security context
                securityContext: {
                  allowPrivilegeEscalation: false,
                  readOnlyRootFilesystem: false,  // Agents need to write build artifacts
                  capabilities: {
                    drop: ['ALL'],
                  },
                },

                // Volume mounts for build workspace
                volumeMounts: [
                  {
                    name: 'workspace',
                    mountPath: '/workspace',
                  },
                  {
                    name: 'tmp',
                    mountPath: '/tmp',
                  },
                ],
              },
            ],

            // Volumes
            volumes: [
              {
                name: 'workspace',
                emptyDir: {
                  sizeLimit: '10Gi',
                },
              },
              {
                name: 'tmp',
                emptyDir: {},
              },
            ],
          },
        },
        // Job configuration
        backoffLimit: 0,  // Don't retry failed jobs
        ttlSecondsAfterFinished: 300,  // Clean up jobs after 5 minutes
      },
    },

    // KEDA ScaledJob
    scaledJob: keda.crds.scaledJob(
      name,
      namespace,
      jobTemplate,
      [
        // Redis Streams trigger
        {
          type: 'redis-streams',
          metadata: {
            address: redisAddress,
            stream: streamName,
            consumerGroup: consumerGroup,
            pendingEntriesCount: std.toString(lagThreshold),
          },
        },
      ],
      pollingInterval,
      maxReplicaCount,
      'default'
    ),
  },
}
