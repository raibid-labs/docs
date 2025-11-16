// gitea.libsonnet - Gitea Helm chart wrapper for Tanka
// Wraps the Gitea chart with OCI registry support for raibid-ci
// Chart: https://gitea.com/gitea/helm-chart

local tanka = import 'github.com/grafana/jsonnet-libs/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);
local config = import '../raibid/config.libsonnet';

{
  // Create Gitea deployment with OCI registry
  // Parameters:
  //   namespace: Kubernetes namespace
  //   name: Release name (default: 'gitea')
  //   domain: Gitea domain (default: 'gitea.local')
  //   values: Additional Helm values to merge
  new(namespace, name='gitea', domain='gitea.local', values={}):: {
    local defaultValues = {
      // Gitea image configuration
      image: {
        repository: 'gitea/gitea',
        tag: '1.21.0',
        pullPolicy: 'IfNotPresent',
      },

      // Single replica for MVP (can scale later)
      replicaCount: 1,

      // Recreate strategy for stateful app
      strategy: {
        type: 'Recreate',
      },

      // Service configuration (NodePort for local dev)
      service: {
        http: {
          type: 'NodePort',
          port: 3000,
          nodePort: 30080,
        },
        ssh: {
          type: 'NodePort',
          port: 22,
          nodePort: 30022,
        },
      },

      // Ingress disabled for MVP
      ingress: {
        enabled: false,
      },

      // Persistence for repositories and data
      persistence: {
        enabled: true,
        size: '10Gi',
        storageClass: 'local-path',
        accessModes: ['ReadWriteOnce'],
      },

      // PostgreSQL database
      postgresql: {
        enabled: true,
        persistence: {
          size: '5Gi',
          storageClass: 'local-path',
        },
        auth: {
          username: 'gitea',
          password: 'gitea',  // Change in production
          database: 'gitea',
        },
      },

      // PostgreSQL HA disabled for MVP
      'postgresql-ha': {
        enabled: false,
      },

      // Internal Redis for caching
      redis: {
        enabled: true,
        architecture: 'standalone',
        auth: {
          enabled: false,
        },
      },

      // Gitea configuration
      gitea: {
        // Admin user
        admin: {
          username: 'raibid-admin',
          email: 'admin@raibid.local',
        },

        // Application config
        config: {
          APP_NAME: 'Raibid CI Git Server',
          RUN_MODE: 'prod',

          // Server configuration
          server: {
            DOMAIN: domain,
            ROOT_URL: 'http://%s:30080/' % domain,
            HTTP_PORT: 3000,
            PROTOCOL: 'http',
            SSH_PORT: 22,
            SSH_LISTEN_PORT: 22,
            DISABLE_SSH: false,
            START_SSH_SERVER: true,
            LFS_START_SERVER: true,
          },

          // Database configuration
          database: {
            DB_TYPE: 'postgres',
            HOST: '%s-postgresql:5432' % name,
            NAME: 'gitea',
            USER: 'gitea',
            PASSWD: 'gitea',  // Change in production
          },

          // Cache configuration
          cache: {
            ADAPTER: 'redis',
            HOST: 'redis://%s-redis-master:6379/0' % name,
          },

          // Session configuration
          session: {
            PROVIDER: 'memory',
          },

          // Webhook configuration
          webhook: {
            ALLOWED_HOST_LIST: '*',
            SKIP_TLS_VERIFY: true,
          },

          // Enable Actions (Gitea CI)
          actions: {
            ENABLED: true,
          },

          // Enable package registry
          packages: {
            ENABLED: true,
          },

          // Enable OCI Container Registry
          'packages.container': {
            ENABLED: true,
          },
        },
      },

      // Resource limits
      resources: {
        requests: {
          cpu: '200m',
          memory: '512Mi',
        },
        limits: {
          cpu: '1000m',
          memory: '2Gi',
        },
      },

      // Security context
      securityContext: {
        runAsUser: 1000,
        runAsGroup: 1000,
        fsGroup: 1000,
      },

      // Health probes
      livenessProbe: {
        enabled: true,
        initialDelaySeconds: 200,
        timeoutSeconds: 5,
        periodSeconds: 10,
        successThreshold: 1,
        failureThreshold: 10,
      },

      readinessProbe: {
        enabled: true,
        initialDelaySeconds: 30,
        timeoutSeconds: 5,
        periodSeconds: 10,
        successThreshold: 1,
        failureThreshold: 3,
      },

      // Common labels
      commonLabels: config.labels.forComponent('gitea') + {
        'app.kubernetes.io/managed-by': 'helm',
      },
    },

    // Merge default values with user-provided values
    local mergedValues = defaultValues + values,

    // Render the Helm chart
    local chart = helm.template(name, '../../vendor/gitea', {
      namespace: namespace,
      values: mergedValues,
      kubeVersion: '1.29',
      includeCrds: true,
    });

    // Return rendered resources
    chart
  },
}
