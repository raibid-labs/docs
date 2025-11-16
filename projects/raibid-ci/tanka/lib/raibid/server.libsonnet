// server.libsonnet - raibid-server deployment configuration
// Kubernetes resources for the API server component

local k = import '../k.libsonnet';
local config = import './config.libsonnet';
local util = import './util.libsonnet';

{
  // Create raibid-server deployment with all supporting resources
  // Parameters:
  //   namespace: Kubernetes namespace
  //   name: Deployment name (default: 'raibid-server')
  //   image: Container image
  //   replicas: Number of replicas (default: 1)
  //   env: Additional environment variables
  new(namespace, name='raibid-server', image='raibid-server:latest', replicas=1, env={}):: {
    local labels = config.labels.forComponent('server'),
    local selector = config.labels.selector('server'),

    // Deployment
    deployment: {
      apiVersion: 'apps/v1',
      kind: 'Deployment',
      metadata: {
        name: name,
        namespace: namespace,
        labels: labels,
      },
      spec: {
        replicas: replicas,
        selector: {
          matchLabels: selector,
        },
        template: {
          metadata: {
            labels: selector,
            annotations: config.annotations.prometheus('8080', '/metrics'),
          },
          spec: {
            // Security context
            securityContext: {
              runAsNonRoot: true,
              runAsUser: 1000,
              fsGroup: 1000,
            },

            containers: [
              {
                name: 'server',
                image: image,
                imagePullPolicy: 'IfNotPresent',

                // Ports
                ports: [
                  {
                    name: 'http',
                    containerPort: config.ports.server.http,
                    protocol: 'TCP',
                  },
                  {
                    name: 'metrics',
                    containerPort: config.ports.server.metrics,
                    protocol: 'TCP',
                  },
                ],

                // Environment variables
                env: [
                  util.env.value('RUST_LOG', 'info'),
                  util.env.value('SERVER_PORT', std.toString(config.ports.server.http)),
                  util.env.value('METRICS_PORT', std.toString(config.ports.server.metrics)),
                  util.env.configMap('REDIS_URL', 'raibid-config', 'REDIS_URL'),
                  util.env.configMap('QUEUE_NAME', 'raibid-config', 'QUEUE_NAME'),
                ] + [
                  util.env.value(k, env[k])
                  for k in std.objectFields(env)
                ],

                // Resource limits
                resources: config.resources.medium,

                // Liveness probe - checks if server is alive
                livenessProbe: {
                  httpGet: {
                    path: '/health',
                    port: 'http',
                  },
                  initialDelaySeconds: 10,
                  periodSeconds: 10,
                  timeoutSeconds: 5,
                  failureThreshold: 3,
                },

                // Readiness probe - checks if server is ready to accept traffic
                readinessProbe: {
                  httpGet: {
                    path: '/health',
                    port: 'http',
                  },
                  initialDelaySeconds: 5,
                  periodSeconds: 5,
                  timeoutSeconds: 3,
                  failureThreshold: 2,
                },

                // Security context
                securityContext: {
                  allowPrivilegeEscalation: false,
                  readOnlyRootFilesystem: true,
                  capabilities: {
                    drop: ['ALL'],
                  },
                },

                // Volume mounts
                volumeMounts: [
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
                name: 'tmp',
                emptyDir: {},
              },
            ],
          },
        },
      },
    },

    // Service
    service: {
      apiVersion: 'v1',
      kind: 'Service',
      metadata: {
        name: config.naming.service('server'),
        namespace: namespace,
        labels: labels,
        annotations: config.annotations.prometheus('8080', '/metrics'),
      },
      spec: {
        type: 'ClusterIP',
        selector: selector,
        ports: [
          {
            name: 'http',
            port: config.ports.server.http,
            targetPort: 'http',
            protocol: 'TCP',
          },
          {
            name: 'metrics',
            port: config.ports.server.metrics,
            targetPort: 'metrics',
            protocol: 'TCP',
          },
        ],
      },
    },
  },
}
