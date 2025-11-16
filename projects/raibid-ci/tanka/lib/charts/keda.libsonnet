// keda.libsonnet - KEDA Helm chart wrapper for Tanka
// Wraps the KEDA chart for event-driven autoscaling in raibid-ci
// Chart: https://github.com/kedacore/charts

local tanka = import 'github.com/grafana/jsonnet-libs/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);
local config = import '../raibid/config.libsonnet';
local k = import '../k.libsonnet';

{
  // Create KEDA deployment with operator and metrics server
  // Parameters:
  //   namespace: Kubernetes namespace
  //   name: Release name (default: 'keda')
  //   values: Additional Helm values to merge
  new(namespace, name='keda', values={}):: {
    local defaultValues = {
      // Operator configuration
      operator: {
        name: 'keda-operator',
        replicaCount: 1,
      },

      // Metrics server configuration
      metricsServer: {
        replicaCount: 1,
      },

      // Admission webhooks
      webhooks: {
        enabled: true,
        port: 9443,
      },

      // Service account
      serviceAccount: {
        create: true,
        name: 'keda-operator',
      },

      // Pod security context
      podSecurityContext: {
        runAsNonRoot: true,
        fsGroup: 1000,
      },

      // Container security context
      securityContext: {
        allowPrivilegeEscalation: false,
        readOnlyRootFilesystem: true,
        capabilities: {
          drop: ['ALL'],
        },
      },

      // Resource limits
      resources: {
        operator: {
          requests: {
            cpu: '100m',
            memory: '128Mi',
          },
          limits: {
            cpu: '200m',
            memory: '256Mi',
          },
        },
        metricServer: {
          requests: {
            cpu: '100m',
            memory: '128Mi',
          },
          limits: {
            cpu: '200m',
            memory: '256Mi',
          },
        },
      },
    },

    // Merge default values with user-provided values
    local mergedValues = defaultValues + values,

    // Render the Helm chart
    local chart = helm.template(name, '../../vendor/keda', {
      namespace: namespace,
      values: mergedValues,
      kubeVersion: '1.29',
      includeCrds: true,
    });

    // Return rendered resources
    chart
  },

  // Helper functions for KEDA CRDs
  crds:: {
    // Create a ScaledJob resource
    // Parameters:
    //   name: ScaledJob name
    //   namespace: Kubernetes namespace
    //   jobTemplate: Kubernetes Job template spec
    //   triggers: Array of KEDA trigger specs
    //   pollingInterval: Polling interval in seconds (default: 10)
    //   maxReplicaCount: Maximum replicas (default: 10)
    //   scalingStrategy: Scaling strategy (default: 'default')
    scaledJob(name, namespace, jobTemplate, triggers, pollingInterval=10, maxReplicaCount=10, scalingStrategy='default'):: {
      apiVersion: 'keda.sh/v1alpha1',
      kind: 'ScaledJob',
      metadata: {
        name: name,
        namespace: namespace,
        labels: config.labels.forComponent('agent'),
      },
      spec: {
        pollingInterval: pollingInterval,
        maxReplicaCount: maxReplicaCount,
        scalingStrategy: {
          strategy: scalingStrategy,
        },
        jobTargetRef: jobTemplate,
        triggers: triggers,
      },
    },

    // Create a TriggerAuthentication resource
    // Parameters:
    //   name: TriggerAuthentication name
    //   namespace: Kubernetes namespace
    //   secretTargetRef: Array of secret references
    triggerAuthentication(name, namespace, secretTargetRef):: {
      apiVersion: 'keda.sh/v1alpha1',
      kind: 'TriggerAuthentication',
      metadata: {
        name: name,
        namespace: namespace,
        labels: config.labels.forComponent('keda'),
      },
      spec: {
        secretTargetRef: secretTargetRef,
      },
    },

    // Redis Streams trigger configuration helper
    // Parameters:
    //   address: Redis server address
    //   stream: Stream name
    //   consumerGroup: Consumer group name
    //   pendingEntriesCount: Threshold for pending entries (default: 5)
    //   authenticationRef: Optional TriggerAuthentication reference
    redisStreamsTrigger(address, stream, consumerGroup, pendingEntriesCount=5, authenticationRef=null):: {
      type: 'redis-streams',
      metadata: {
        addressFromEnv: 'REDIS_ADDRESS',
        stream: stream,
        consumerGroup: consumerGroup,
        pendingEntriesCount: std.toString(pendingEntriesCount),
      },
      [if authenticationRef != null then 'authenticationRef']: {
        name: authenticationRef,
      },
    },
  },
}
