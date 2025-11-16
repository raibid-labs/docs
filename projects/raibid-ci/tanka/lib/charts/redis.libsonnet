// redis.libsonnet - Redis Helm chart wrapper for Tanka
// Wraps the Bitnami Redis chart with opinionated defaults for raibid-ci
// Chart: https://github.com/bitnami/charts/tree/main/bitnami/redis

local tanka = import 'github.com/grafana/jsonnet-libs/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);
local config = import '../raibid/config.libsonnet';

{
  // Create Redis deployment with Streams support
  // Parameters:
  //   namespace: Kubernetes namespace
  //   name: Release name (default: 'redis')
  //   values: Additional Helm values to merge
  new(namespace, name='redis', values={}):: {
    local defaultValues = {
      // Architecture: standalone for MVP (can upgrade to replication later)
      architecture: 'standalone',

      // Authentication (disabled for MVP, enable in production)
      auth: {
        enabled: true,
        password: '',  // Will be auto-generated if empty
      },

      // Master/Primary configuration
      master: {
        persistence: {
          enabled: true,
          size: '8Gi',
          storageClass: 'local-path',
          accessModes: ['ReadWriteOnce'],
        },

        // Resource limits - optimized for job queue workload
        resources: {
          requests: {
            cpu: '100m',
            memory: '256Mi',
          },
          limits: {
            cpu: '500m',
            memory: '512Mi',
          },
        },

        // Security context
        podSecurityContext: {
          enabled: true,
          fsGroup: 1001,
          runAsUser: 1001,
        },

        // Health probes
        livenessProbe: {
          enabled: true,
          initialDelaySeconds: 20,
          periodSeconds: 10,
          timeoutSeconds: 5,
          successThreshold: 1,
          failureThreshold: 5,
        },

        readinessProbe: {
          enabled: true,
          initialDelaySeconds: 20,
          periodSeconds: 10,
          timeoutSeconds: 5,
          successThreshold: 1,
          failureThreshold: 5,
        },
      },

      // Redis configuration optimized for Streams
      commonConfiguration: |||
        # Persistence settings
        appendonly yes
        appendfsync everysec

        # Memory management
        maxmemory 450mb
        maxmemory-policy allkeys-lru

        # Redis Streams configuration
        stream-node-max-entries 100

        # Logging
        loglevel notice
      |||,

      // Metrics for monitoring
      metrics: {
        enabled: true,
        serviceMonitor: {
          enabled: false,  // Enable with Prometheus Operator
        },
      },

      // Network policy (disabled for MVP)
      networkPolicy: {
        enabled: false,
      },

      // Common labels
      commonLabels: config.labels.forComponent('redis') + {
        'app.kubernetes.io/managed-by': 'helm',
      },
    },

    // Merge default values with user-provided values
    local mergedValues = defaultValues + values,

    // Render the Helm chart
    local chart = helm.template(name, '../../vendor/redis', {
      namespace: namespace,
      values: mergedValues,
      kubeVersion: '1.29',
      includeCrds: true,
    });

    // Return rendered resources
    chart
  },
}
