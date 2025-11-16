// Central configuration for MOP (Managed Observability Platform)
// This file provides environment-specific and component-specific configuration

{
  // Environment configurations
  environments:: {
    dev: {
      name: 'dev',
      namespace: 'observability-dev',
      domain: 'dev.mop.local',
      replicas: {
        alloy: 1,
        tempo: 1,
        mimir: 1,
        loki: 1,
        grafana: 1,
      },
      resources: {
        // Dev uses minimal resources
        alloy: {
          requests: { cpu: '100m', memory: '256Mi' },
          limits: { cpu: '500m', memory: '512Mi' },
        },
        tempo: {
          requests: { cpu: '100m', memory: '512Mi' },
          limits: { cpu: '1', memory: '2Gi' },
        },
        mimir: {
          requests: { cpu: '200m', memory: '1Gi' },
          limits: { cpu: '2', memory: '4Gi' },
        },
        loki: {
          requests: { cpu: '100m', memory: '512Mi' },
          limits: { cpu: '1', memory: '2Gi' },
        },
        grafana: {
          requests: { cpu: '50m', memory: '128Mi' },
          limits: { cpu: '200m', memory: '256Mi' },
        },
      },
      storage: {
        class: 'standard',
        type: 'filesystem',  // Use local filesystem in dev
      },
    },

    staging: {
      name: 'staging',
      namespace: 'observability-staging',
      domain: 'staging.mop.local',
      replicas: {
        alloy: 2,
        tempo: 2,
        mimir: 3,
        loki: 2,
        grafana: 2,
      },
      resources: {
        // Staging uses moderate resources
        alloy: {
          requests: { cpu: '500m', memory: '1Gi' },
          limits: { cpu: '2', memory: '2Gi' },
        },
        tempo: {
          requests: { cpu: '500m', memory: '2Gi' },
          limits: { cpu: '2', memory: '4Gi' },
        },
        mimir: {
          requests: { cpu: '1', memory: '4Gi' },
          limits: { cpu: '4', memory: '8Gi' },
        },
        loki: {
          requests: { cpu: '500m', memory: '2Gi' },
          limits: { cpu: '2', memory: '4Gi' },
        },
        grafana: {
          requests: { cpu: '100m', memory: '256Mi' },
          limits: { cpu: '500m', memory: '512Mi' },
        },
      },
      storage: {
        class: 'fast-ssd',
        type: 's3',
        s3: {
          endpoint: 's3.amazonaws.com',
          bucket: 'mop-staging',
        },
      },
    },

    production: {
      name: 'production',
      namespace: 'observability',
      domain: 'mop.example.com',
      replicas: {
        alloy: 3,
        tempo: 3,
        mimir: 3,
        loki: 3,
        grafana: 2,
      },
      resources: {
        // Production uses full resources
        alloy: {
          requests: { cpu: '1', memory: '2Gi' },
          limits: { cpu: '4', memory: '4Gi' },
        },
        tempo: {
          requests: { cpu: '2', memory: '4Gi' },
          limits: { cpu: '8', memory: '16Gi' },
        },
        mimir: {
          requests: { cpu: '4', memory: '8Gi' },
          limits: { cpu: '16', memory: '32Gi' },
        },
        loki: {
          requests: { cpu: '2', memory: '4Gi' },
          limits: { cpu: '8', memory: '16Gi' },
        },
        grafana: {
          requests: { cpu: '500m', memory: '1Gi' },
          limits: { cpu: '2', memory: '2Gi' },
        },
      },
      storage: {
        class: 'fast-ssd',
        type: 's3',
        s3: {
          endpoint: 's3.amazonaws.com',
          bucket: 'mop-production',
          region: 'us-east-1',
        },
      },
    },
  },

  // Component versions
  versions:: {
    obi: '0.1.0',
    alloy: '1.0.0',
    tempo: '2.3.1',
    mimir: '5.3.0',
    loki: '5.41.0',
    grafana: '10.2.3',
  },

  // Helm chart repositories
  helmRepos:: {
    grafana: 'https://grafana.github.io/helm-charts',
    opentelemetry: 'https://open-telemetry.github.io/opentelemetry-helm-charts',
  },

  // Common labels applied to all resources
  commonLabels:: {
    'app.kubernetes.io/managed-by': 'tanka',
    'app.kubernetes.io/part-of': 'mop',
  },

  // OBI configuration
  obi:: {
    protocols: ['http', 'grpc', 'sql', 'redis', 'kafka'],
    export: {
      protocol: 'otlp',
      endpoint: 'alloy.observability.svc.cluster.local:4317',
    },
    resources: {
      requests: { cpu: '50m', memory: '64Mi' },
      limits: { cpu: '200m', memory: '256Mi' },
    },
  },

  // Alloy configuration
  alloy:: {
    receivers: {
      otlp: {
        grpc: { endpoint: '0.0.0.0:4317' },
        http: { endpoint: '0.0.0.0:4318' },
      },
    },
    processors: {
      batch: {
        timeout: '10s',
        send_batch_size: 1024,
      },
      memory_limiter: {
        check_interval: '1s',
        limit_mib: 1024,
      },
    },
    exporters: {
      tempo: 'tempo.observability.svc.cluster.local:4317',
      mimir: 'mimir.observability.svc.cluster.local:9009',
      loki: 'loki.observability.svc.cluster.local:3100',
    },
  },

  // Tempo configuration
  tempo:: {
    retention: {
      traces: '720h',  // 30 days
    },
    ingestion: {
      max_bytes_per_trace: 5000000,  // 5MB
      rate_limit_bytes: 15000000,    // 15MB/s
    },
    query: {
      max_duration: '0s',  // No limit
    },
  },

  // Mimir configuration
  mimir:: {
    retention: {
      metrics: '8760h',  // 365 days
    },
    limits: {
      ingestion_rate: 10000,
      ingestion_burst_size: 200000,
      max_global_series_per_user: 10000000,
    },
    blocks_storage: {
      retention_period: '30d',
    },
  },

  // Loki configuration
  loki:: {
    retention: {
      logs: '720h',  // 30 days
    },
    limits: {
      ingestion_rate_mb: 10,
      ingestion_burst_size_mb: 20,
      max_streams_per_user: 10000,
      max_query_length: '721h',
    },
  },

  // Grafana configuration
  grafana:: {
    auth: {
      disable_login_form: false,
      anonymous: {
        enabled: true,
        org_role: 'Admin',
      },
    },
    datasources: [
      {
        name: 'Tempo',
        type: 'tempo',
        url: 'http://tempo.observability.svc.cluster.local:3200',
        isDefault: false,
        jsonData: {
          tracesToLogs: {
            datasourceUid: 'loki',
            tags: ['job', 'instance', 'pod'],
          },
          tracesToMetrics: {
            datasourceUid: 'mimir',
          },
          serviceMap: {
            datasourceUid: 'mimir',
          },
        },
      },
      {
        name: 'Mimir',
        type: 'prometheus',
        url: 'http://mimir.observability.svc.cluster.local:9009/prometheus',
        isDefault: true,
        jsonData: {
          timeInterval: '15s',
        },
      },
      {
        name: 'Loki',
        type: 'loki',
        url: 'http://loki.observability.svc.cluster.local:3100',
        isDefault: false,
        jsonData: {
          derivedFields: [
            {
              datasourceUid: 'tempo',
              matcherRegex: 'trace_id=(\\w+)',
              name: 'TraceID',
              url: '$${__value.raw}',
            },
          ],
        },
      },
    ],
  },
}
