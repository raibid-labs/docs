// helm.libsonnet - Helm chart wrappers for raibid-ci
// Provides helpers for rendering and customizing Helm charts with Tanka

local tanka = import 'github.com/grafana/jsonnet-libs/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);

{
  // Render a Helm chart with custom values
  // Usage:
  //   local redis = helm.render('redis', 'charts/redis', {...})
  render(name, chartPath, values={}, namespace='default', kubeVersion='1.29')::
    helm.template(name, chartPath, {
      values: values,
      namespace: namespace,
      kubeVersion: kubeVersion,
      noHooks: false,
      includeCrds: true,
    }),

  // Common chart configurations
  charts:: {
    // Redis chart configuration
    // https://github.com/bitnami/charts/tree/main/bitnami/redis
    redis(namespace, replicas=1, persistence=true, storageClass='local-path'):: {
      local chartPath = '../../vendor/redis',  // Adjust based on vendored location

      architecture: 'standalone',
      auth: {
        enabled: false,  // Development only
      },
      master: {
        count: replicas,
        persistence: {
          enabled: persistence,
          storageClass: storageClass,
          size: '8Gi',
        },
        resources: {
          requests: { memory: '256Mi', cpu: '250m' },
          limits: { memory: '512Mi', cpu: '500m' },
        },
      },
      // Enable Redis Streams
      commonConfiguration: |||
        # Redis Streams configuration
        stream-node-max-entries 1000
        stream-node-max-bytes 4096
      |||,
      metrics: {
        enabled: true,
        serviceMonitor: {
          enabled: false,  // Enable with Prometheus Operator
        },
      },
    },

    // Gitea chart configuration
    // https://gitea.com/gitea/helm-chart
    gitea(namespace, domain='gitea.local', storageClass='local-path'):: {
      local chartPath = '../../vendor/gitea',

      service: {
        http: {
          type: 'ClusterIP',
          port: 3000,
        },
        ssh: {
          type: 'ClusterIP',
          port: 2222,
        },
      },
      ingress: {
        enabled: false,  // Use port-forward for local dev
      },
      persistence: {
        enabled: true,
        storageClass: storageClass,
        size: '10Gi',
      },
      gitea: {
        admin: {
          username: 'gitea_admin',
          password: 'changeme',  // Override via secret in production
          email: 'admin@' + domain,
        },
        config: {
          server: {
            DOMAIN: domain,
            ROOT_URL: 'http://' + domain,
          },
          database: {
            DB_TYPE: 'sqlite3',  // Use postgres in production
          },
          repository: {
            DEFAULT_BRANCH: 'main',
          },
          // Enable OCI registry
          packages: {
            ENABLED: true,
          },
        },
      },
      resources: {
        requests: { memory: '512Mi', cpu: '500m' },
        limits: { memory: '1Gi', cpu: '1000m' },
      },
    },

    // KEDA chart configuration
    // https://github.com/kedacore/charts
    keda(namespace):: {
      local chartPath = '../../vendor/keda',

      operator: {
        name: 'keda-operator',
        replicaCount: 1,
      },
      metricsServer: {
        replicaCount: 1,
      },
      webhooks: {
        enabled: true,
        port: 9443,
      },
      serviceAccount: {
        create: true,
        name: 'keda-operator',
      },
      podSecurityContext: {
        runAsNonRoot: true,
        fsGroup: 1000,
      },
      securityContext: {
        allowPrivilegeEscalation: false,
        readOnlyRootFilesystem: true,
        capabilities: {
          drop: ['ALL'],
        },
      },
      resources: {
        operator: {
          requests: { memory: '128Mi', cpu: '100m' },
          limits: { memory: '256Mi', cpu: '200m' },
        },
        metricServer: {
          requests: { memory: '128Mi', cpu: '100m' },
          limits: { memory: '256Mi', cpu: '200m' },
        },
      },
    },

    // Flux chart configuration
    // https://github.com/fluxcd-community/helm-charts
    flux(namespace, gitUrl=null, gitBranch='main'):: {
      local chartPath = '../../vendor/flux',

      sourceController: {
        create: true,
      },
      kustomizeController: {
        create: true,
      },
      helmController: {
        create: true,
      },
      notificationController: {
        create: false,  // Enable for notifications
      },
      imageAutomationController: {
        create: false,  // Enable for image automation
      },
      imageReflectorController: {
        create: false,  // Enable for image scanning
      },
      policies: {
        create: false,  // Network policies
      },
      rbac: {
        create: true,
      },
      // Git repository configuration
      [if gitUrl != null then 'gitRepository']: {
        spec: {
          url: gitUrl,
          ref: {
            branch: gitBranch,
          },
          interval: '1m',
        },
      },
    },
  },

  // Helper to customize Helm rendered resources
  customize:: {
    // Add labels to all resources
    addLabels(resources, labels)::
      std.mapWithKey(
        function(k, v) v {
          metadata+: {
            labels+: labels,
          },
        },
        resources
      ),

    // Add annotations to all resources
    addAnnotations(resources, annotations)::
      std.mapWithKey(
        function(k, v) v {
          metadata+: {
            annotations+: annotations,
          },
        },
        resources
      ),

    // Override resource limits
    overrideResources(resources, containerName, requests, limits)::
      std.mapWithKey(
        function(k, v)
          if std.objectHas(v, 'spec') && std.objectHas(v.spec, 'template') then
            v {
              spec+: {
                template+: {
                  spec+: {
                    containers: [
                      if c.name == containerName then
                        c {
                          resources: {
                            requests: requests,
                            limits: limits,
                          },
                        }
                      else c
                      for c in super.containers
                    ],
                  },
                },
              },
            }
          else v,
        resources
      ),

    // Change service type
    changeServiceType(resources, serviceName, type)::
      std.mapWithKey(
        function(k, v)
          if v.kind == 'Service' && v.metadata.name == serviceName then
            v {
              spec+: {
                type: type,
              },
            }
          else v,
        resources
      ),
  },
}
