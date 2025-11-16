// config.libsonnet - Configuration helpers for raibid-ci
// Provides standard naming, labeling, and namespace conventions
{
  // Project-wide configuration
  project: {
    name: 'raibid-ci',
    component: 'ci-system',
    version: '0.1.0',
  },

  // Default namespace for all raibid components
  namespace: 'raibid-system',

  // Standard labels applied to all resources
  // https://kubernetes.io/docs/concepts/overview/working-with-objects/common-labels/
  labels:: {
    common: {
      'app.kubernetes.io/name': $.project.name,
      'app.kubernetes.io/component': $.project.component,
      'app.kubernetes.io/version': $.project.version,
      'app.kubernetes.io/managed-by': 'tanka',
    },

    // Generate labels for a specific component
    // Usage: config.labels.forComponent('server')
    forComponent(component):: $.labels.common + {
      'app.kubernetes.io/component': component,
      'app.kubernetes.io/part-of': $.project.name,
    },

    // Add instance-specific labels
    // Usage: config.labels.forInstance('server', 'api')
    forInstance(component, instance):: $.labels.forComponent(component) + {
      'app.kubernetes.io/instance': instance,
    },

    // Selector labels (subset of all labels, used for matchLabels)
    // These should be immutable across updates
    selector(component):: {
      'app.kubernetes.io/name': $.project.name,
      'app.kubernetes.io/component': component,
    },
  },

  // Standard annotations
  annotations:: {
    common: {
      'raibid.dev/managed': 'true',
    },

    // Add prometheus scraping annotations
    prometheus(port='8080', path='/metrics'):: {
      'prometheus.io/scrape': 'true',
      'prometheus.io/port': port,
      'prometheus.io/path': path,
    },
  },

  // Naming conventions
  naming:: {
    // Generate standard resource name
    // Usage: config.naming.resource('server', 'deployment')
    resource(component, kind):: '%s-%s' % [component, kind],

    // Generate config resource name
    config(component, suffix='config'):: '%s-%s' % [component, suffix],

    // Generate secret name
    secret(component, suffix='secret'):: '%s-%s' % [component, suffix],

    // Service name (matches component name)
    service(component):: component,

    // Headless service name
    headless(component):: '%s-headless' % component,
  },

  // Environment-specific overrides
  environments:: {
    'local': {
      namespace: 'raibid-system',
      domain: 'local.raibid.dev',
      registry: 'gitea.raibid-system.svc.cluster.local:5000',
    },
    staging: {
      namespace: 'raibid-staging',
      domain: 'staging.raibid.dev',
      registry: 'registry.raibid.dev',
    },
    production: {
      namespace: 'raibid-prod',
      domain: 'raibid.dev',
      registry: 'registry.raibid.dev',
    },
  },

  // Resource limits and requests
  resources:: {
    // Tiny: For init containers, sidecars
    tiny: {
      requests: { memory: '32Mi', cpu: '10m' },
      limits: { memory: '64Mi', cpu: '50m' },
    },

    // Small: For lightweight services
    small: {
      requests: { memory: '128Mi', cpu: '100m' },
      limits: { memory: '256Mi', cpu: '200m' },
    },

    // Medium: For typical services (server, agent)
    medium: {
      requests: { memory: '512Mi', cpu: '500m' },
      limits: { memory: '1Gi', cpu: '1000m' },
    },

    // Large: For resource-intensive services
    large: {
      requests: { memory: '2Gi', cpu: '2000m' },
      limits: { memory: '4Gi', cpu: '4000m' },
    },
  },

  // Common ports
  ports:: {
    server: {
      http: 8080,
      grpc: 9090,
      metrics: 8081,
    },
    redis: {
      redis: 6379,
    },
    gitea: {
      http: 3000,
      ssh: 2222,
    },
    keda: {
      metrics: 9022,
      webhook: 9443,
    },
  },
}
