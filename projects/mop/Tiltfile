# -*- mode: Python -*-

"""
MOP (Managed Observability Platform) - Tiltfile
Local development environment with hot reload for Kubernetes deployments
"""

# ==============================================================================
# CONFIGURATION
# ==============================================================================

config.define_string_list('to-run', args=True)
cfg = config.parse()
groups = {
    'infra': ['namespace', 'helm-repos'],
    'storage': ['tempo', 'mimir', 'loki'],
    'pipeline': ['obi', 'alloy'],
    'viz': ['grafana'],
}

# ==============================================================================
# KUBERNETES CONTEXT VALIDATION
# ==============================================================================

# Allow specific Kubernetes contexts for safety
allowed_contexts = [
    'kind-mop',
    'kind-mop-dev',
    'minikube',
    'docker-desktop',
    'rancher-desktop',
]

k8s_context = str(local('kubectl config current-context', quiet=True, echo_off=True)).strip()

if k8s_context not in allowed_contexts:
    fail("""
    ❌ Invalid Kubernetes context: {}

    Allowed contexts: {}

    To create a kind cluster for MOP:
      kind create cluster --name mop

    To switch context:
      kubectl config use-context kind-mop
    """.format(k8s_context, ', '.join(allowed_contexts)))

print('✅ Using Kubernetes context: {}'.format(k8s_context))

# ==============================================================================
# EXTENSIONS
# ==============================================================================

load('ext://helm_resource', 'helm_resource', 'helm_repo')
load('ext://namespace', 'namespace_create', 'namespace_inject')
load('ext://restart_process', 'docker_build_with_restart')

# ==============================================================================
# HELPER FUNCTIONS
# ==============================================================================

def resource_group(name, deps):
    """Create a resource group for organizing dependencies"""
    return {
        'name': name,
        'deps': deps
    }

# ==============================================================================
# NAMESPACE CREATION
# ==============================================================================

namespace_create('observability')
print('✅ Namespace: observability')

# ==============================================================================
# HELM REPOSITORY MANAGEMENT
# ==============================================================================

helm_repo('grafana', 'https://grafana.github.io/helm-charts', labels=['helm-repos'])
helm_repo('open-telemetry', 'https://open-telemetry.github.io/opentelemetry-helm-charts', labels=['helm-repos'])

# Update Helm repos
local_resource(
    'helm-repo-update',
    'helm repo update',
    deps=[],
    labels=['infra', 'helm-repos'],
    resource_deps=['grafana', 'open-telemetry']
)

print('✅ Helm repositories configured')

# ==============================================================================
# TEMPO - DISTRIBUTED TRACING
# ==============================================================================

# Tempo configuration values (dev-optimized)
tempo_values = {
    'tempo': {
        'repository': 'grafana/tempo',
        'tag': '2.3.1',
        'pullPolicy': 'IfNotPresent'
    },
    'tempoQuery': {
        'enabled': True,
        'repository': 'grafana/tempo-query',
        'tag': '2.3.1'
    },
    'persistence': {
        'enabled': False  # Use emptyDir for dev
    },
    'resources': {
        'requests': {
            'cpu': '100m',
            'memory': '128Mi'
        },
        'limits': {
            'cpu': '500m',
            'memory': '512Mi'
        }
    },
    'config': {
        'auth_enabled': False,
        'server': {
            'http_listen_port': 3200
        },
        'distributor': {
            'receivers': {
                'otlp': {
                    'protocols': {
                        'grpc': {
                            'endpoint': '0.0.0.0:4317'
                        },
                        'http': {
                            'endpoint': '0.0.0.0:4318'
                        }
                    }
                }
            }
        },
        'ingester': {
            'trace_idle_period': '30s',
            'max_block_duration': '1m'
        },
        'compactor': {
            'compaction': {
                'block_retention': '24h'
            }
        },
        'storage': {
            'trace': {
                'backend': 'local',
                'local': {
                    'path': '/var/tempo/traces'
                }
            }
        }
    }
}

helm_resource(
    'tempo',
    'grafana/tempo',
    namespace='observability',
    flags=[
        '--set-json', 'tempo={}'.format(encode_json(tempo_values['tempo'])),
        '--set-json', 'tempoQuery={}'.format(encode_json(tempo_values['tempoQuery'])),
        '--set-json', 'persistence={}'.format(encode_json(tempo_values['persistence'])),
        '--set-json', 'resources={}'.format(encode_json(tempo_values['resources'])),
        '--set-json', 'config={}'.format(encode_json(tempo_values['config'])),
    ],
    labels=['storage', 'tempo'],
    resource_deps=['helm-repo-update'],
    port_forwards=['3200:3200', '4317:4317', '4318:4318']
)

print('✅ Tempo configured (traces)')

# ==============================================================================
# MIMIR - METRICS STORAGE
# ==============================================================================

# Mimir configuration values (dev-optimized, monolithic mode)
mimir_values = {
    'mimir': {
        'structuredConfig': {
            'multitenancy_enabled': False,
            'blocks_storage': {
                'backend': 'filesystem',
                'filesystem': {
                    'dir': '/data/mimir-blocks'
                }
            },
            'ruler_storage': {
                'backend': 'filesystem',
                'filesystem': {
                    'dir': '/data/mimir-ruler'
                }
            },
            'alertmanager_storage': {
                'backend': 'filesystem',
                'filesystem': {
                    'dir': '/data/mimir-alertmanager'
                }
            }
        }
    },
    'minio': {
        'enabled': False  # Use filesystem for dev
    },
    'nginx': {
        'enabled': False  # Direct access for dev
    },
    'gateway': {
        'enabled': False
    },
    'compactor': {
        'replicas': 1,
        'resources': {
            'requests': {
                'cpu': '100m',
                'memory': '128Mi'
            },
            'limits': {
                'cpu': '500m',
                'memory': '512Mi'
            }
        }
    },
    'distributor': {
        'replicas': 1,
        'resources': {
            'requests': {
                'cpu': '100m',
                'memory': '128Mi'
            },
            'limits': {
                'cpu': '500m',
                'memory': '512Mi'
            }
        }
    },
    'ingester': {
        'replicas': 1,
        'resources': {
            'requests': {
                'cpu': '100m',
                'memory': '256Mi'
            },
            'limits': {
                'cpu': '500m',
                'memory': '1Gi'
            }
        },
        'persistentVolume': {
            'enabled': False  # Use emptyDir for dev
        }
    },
    'querier': {
        'replicas': 1,
        'resources': {
            'requests': {
                'cpu': '100m',
                'memory': '128Mi'
            },
            'limits': {
                'cpu': '500m',
                'memory': '512Mi'
            }
        }
    },
    'query_frontend': {
        'replicas': 1,
        'resources': {
            'requests': {
                'cpu': '100m',
                'memory': '128Mi'
            },
            'limits': {
                'cpu': '500m',
                'memory': '512Mi'
            }
        }
    },
    'store_gateway': {
        'replicas': 1,
        'resources': {
            'requests': {
                'cpu': '100m',
                'memory': '128Mi'
            },
            'limits': {
                'cpu': '500m',
                'memory': '512Mi'
            }
        },
        'persistentVolume': {
            'enabled': False
        }
    }
}

helm_resource(
    'mimir',
    'grafana/mimir-distributed',
    namespace='observability',
    flags=[
        '--set-json', 'mimir={}'.format(encode_json(mimir_values['mimir'])),
        '--set', 'minio.enabled=false',
        '--set', 'nginx.enabled=false',
        '--set', 'gateway.enabled=false',
        '--set', 'compactor.replicas=1',
        '--set', 'distributor.replicas=1',
        '--set', 'ingester.replicas=1',
        '--set', 'querier.replicas=1',
        '--set', 'query_frontend.replicas=1',
        '--set', 'store_gateway.replicas=1',
        '--set', 'ingester.persistentVolume.enabled=false',
        '--set', 'store_gateway.persistentVolume.enabled=false',
    ],
    labels=['storage', 'mimir'],
    resource_deps=['helm-repo-update'],
    port_forwards=['9009:8080']  # Mimir query frontend
)

print('✅ Mimir configured (metrics)')

# ==============================================================================
# LOKI - LOG AGGREGATION
# ==============================================================================

# Loki configuration values (dev-optimized, single binary)
loki_values = {
    'loki': {
        'auth_enabled': False,
        'server': {
            'http_listen_port': 3100
        },
        'commonConfig': {
            'replication_factor': 1
        },
        'storage': {
            'type': 'filesystem'
        }
    },
    'singleBinary': {
        'replicas': 1,
        'resources': {
            'requests': {
                'cpu': '100m',
                'memory': '256Mi'
            },
            'limits': {
                'cpu': '500m',
                'memory': '512Mi'
            }
        },
        'persistence': {
            'enabled': False  # Use emptyDir for dev
        }
    },
    'read': {
        'replicas': 0  # Disable read/write split for dev
    },
    'write': {
        'replicas': 0
    },
    'backend': {
        'replicas': 0
    },
    'monitoring': {
        'selfMonitoring': {
            'enabled': False
        },
        'lokiCanary': {
            'enabled': False
        }
    },
    'test': {
        'enabled': False
    }
}

helm_resource(
    'loki',
    'grafana/loki',
    namespace='observability',
    flags=[
        '--set-json', 'loki={}'.format(encode_json(loki_values['loki'])),
        '--set', 'singleBinary.replicas=1',
        '--set', 'singleBinary.persistence.enabled=false',
        '--set', 'read.replicas=0',
        '--set', 'write.replicas=0',
        '--set', 'backend.replicas=0',
        '--set', 'monitoring.selfMonitoring.enabled=false',
        '--set', 'monitoring.lokiCanary.enabled=false',
        '--set', 'test.enabled=false',
    ],
    labels=['storage', 'loki'],
    resource_deps=['helm-repo-update'],
    port_forwards=['3100:3100']
)

print('✅ Loki configured (logs)')

# ==============================================================================
# GRAFANA ALLOY - TELEMETRY PIPELINE
# ==============================================================================

# Alloy configuration for dev (standalone mode, not operator)
alloy_config = """
// Prometheus scraping
prometheus.scrape "default" {
  targets = [
    {"__address__" = "localhost:12345"},
  ]
  forward_to = [prometheus.remote_write.mimir.receiver]
}

// Remote write to Mimir
prometheus.remote_write "mimir" {
  endpoint {
    url = "http://mimir-distributor.observability.svc.cluster.local:8080/api/v1/push"
  }
}

// OTLP receiver for traces and metrics
otelcol.receiver.otlp "default" {
  grpc {
    endpoint = "0.0.0.0:4317"
  }
  http {
    endpoint = "0.0.0.0:4318"
  }

  output {
    traces  = [otelcol.processor.batch.default.input]
    metrics = [otelcol.processor.batch.default.input]
  }
}

// Batch processor
otelcol.processor.batch "default" {
  output {
    traces  = [otelcol.exporter.otlp.tempo.input]
    metrics = [otelcol.exporter.prometheus.mimir.input]
  }
}

// Export traces to Tempo
otelcol.exporter.otlp "tempo" {
  client {
    endpoint = "tempo.observability.svc.cluster.local:4317"
    tls {
      insecure = true
    }
  }
}

// Export metrics to Mimir via Prometheus remote write
otelcol.exporter.prometheus "mimir" {
  forward_to = [prometheus.remote_write.mimir.receiver]
}

// Loki receiver for logs
loki.source.api "default" {
  http {
    listen_address = "0.0.0.0"
    listen_port    = 3100
  }

  forward_to = [loki.write.default.receiver]
}

// Write logs to Loki
loki.write "default" {
  endpoint {
    url = "http://loki.observability.svc.cluster.local:3100/loki/api/v1/push"
  }
}
"""

# Write Alloy config to temporary file
local('mkdir -p /tmp/mop-tilt')
local('cat > /tmp/mop-tilt/alloy-config.alloy << EOF\n{}\nEOF'.format(alloy_config))

# Create ConfigMap for Alloy
k8s_yaml(blob("""
apiVersion: v1
kind: ConfigMap
metadata:
  name: alloy-config
  namespace: observability
data:
  config.alloy: |
{}
""".format('\n'.join(['    ' + line for line in alloy_config.split('\n')]))))

# Deploy Alloy
alloy_values = {
    'alloy': {
        'configMap': {
            'create': False,
            'name': 'alloy-config',
            'key': 'config.alloy'
        }
    },
    'controller': {
        'type': 'deployment',
        'replicas': 1
    },
    'resources': {
        'requests': {
            'cpu': '100m',
            'memory': '128Mi'
        },
        'limits': {
            'cpu': '500m',
            'memory': '512Mi'
        }
    }
}

helm_resource(
    'alloy',
    'grafana/alloy',
    namespace='observability',
    flags=[
        '--set', 'alloy.configMap.create=false',
        '--set', 'alloy.configMap.name=alloy-config',
        '--set', 'alloy.configMap.key=config.alloy',
        '--set', 'controller.type=deployment',
        '--set', 'controller.replicas=1',
    ],
    labels=['pipeline', 'alloy'],
    resource_deps=['helm-repo-update', 'tempo', 'mimir', 'loki'],
    port_forwards=['12345:12345', '4317:4317', '4318:4318']
)

print('✅ Alloy configured (telemetry pipeline)')

# ==============================================================================
# OBI - eBPF INSTRUMENTATION
# ==============================================================================

# OBI DaemonSet configuration
obi_yaml = """
apiVersion: v1
kind: ServiceAccount
metadata:
  name: obi
  namespace: observability
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: obi
rules:
- apiGroups: [""]
  resources: ["pods", "nodes"]
  verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: obi
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: obi
subjects:
- kind: ServiceAccount
  name: obi
  namespace: observability
---
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: obi
  namespace: observability
  labels:
    app: obi
spec:
  selector:
    matchLabels:
      app: obi
  template:
    metadata:
      labels:
        app: obi
    spec:
      serviceAccountName: obi
      hostPID: true
      hostNetwork: true
      containers:
      - name: obi
        image: ghcr.io/open-telemetry/opentelemetry-ebpf:latest
        imagePullPolicy: IfNotPresent
        env:
        - name: OTEL_EXPORTER_OTLP_ENDPOINT
          value: "http://alloy.observability.svc.cluster.local:4317"
        - name: OTEL_SERVICE_NAME
          value: "obi-agent"
        - name: OTEL_RESOURCE_ATTRIBUTES
          value: "deployment.environment=dev"
        - name: NODE_NAME
          valueFrom:
            fieldRef:
              fieldPath: spec.nodeName
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        securityContext:
          privileged: true
          capabilities:
            add:
            - SYS_ADMIN
            - SYS_PTRACE
            - NET_ADMIN
        resources:
          requests:
            cpu: 50m
            memory: 64Mi
          limits:
            cpu: 200m
            memory: 256Mi
        volumeMounts:
        - name: sys
          mountPath: /sys
          readOnly: true
        - name: debugfs
          mountPath: /sys/kernel/debug
      volumes:
      - name: sys
        hostPath:
          path: /sys
      - name: debugfs
        hostPath:
          path: /sys/kernel/debug
"""

k8s_yaml(blob(obi_yaml))

k8s_resource(
    'obi',
    labels=['pipeline', 'obi'],
    resource_deps=['alloy'],
    objects=['obi:serviceaccount', 'obi:clusterrole', 'obi:clusterrolebinding']
)

print('✅ OBI configured (eBPF instrumentation)')

# ==============================================================================
# GRAFANA - VISUALIZATION
# ==============================================================================

# Grafana datasources configuration
grafana_datasources = {
    'datasources.yaml': {
        'apiVersion': 1,
        'datasources': [
            {
                'name': 'Tempo',
                'type': 'tempo',
                'access': 'proxy',
                'url': 'http://tempo.observability.svc.cluster.local:3200',
                'isDefault': False,
                'editable': True
            },
            {
                'name': 'Mimir',
                'type': 'prometheus',
                'access': 'proxy',
                'url': 'http://mimir-query-frontend.observability.svc.cluster.local:8080/prometheus',
                'isDefault': True,
                'editable': True,
                'jsonData': {
                    'timeInterval': '30s'
                }
            },
            {
                'name': 'Loki',
                'type': 'loki',
                'access': 'proxy',
                'url': 'http://loki.observability.svc.cluster.local:3100',
                'isDefault': False,
                'editable': True,
                'jsonData': {
                    'derivedFields': [
                        {
                            'datasourceUid': 'tempo',
                            'matcherRegex': 'traceID=(\\w+)',
                            'name': 'TraceID',
                            'url': '$${__value.raw}'
                        }
                    ]
                }
            }
        ]
    }
}

grafana_values = {
    'replicas': 1,
    'adminUser': 'admin',
    'adminPassword': 'admin',
    'service': {
        'type': 'ClusterIP',
        'port': 3000
    },
    'resources': {
        'requests': {
            'cpu': '100m',
            'memory': '128Mi'
        },
        'limits': {
            'cpu': '500m',
            'memory': '512Mi'
        }
    },
    'persistence': {
        'enabled': False  # Stateless for dev
    },
    'datasources': grafana_datasources,
    'env': {
        'GF_AUTH_ANONYMOUS_ENABLED': 'true',
        'GF_AUTH_ANONYMOUS_ORG_ROLE': 'Admin',
        'GF_AUTH_DISABLE_LOGIN_FORM': 'true',
        'GF_SECURITY_ALLOW_EMBEDDING': 'true',
        'GF_INSTALL_PLUGINS': ''
    },
    'dashboardProviders': {
        'dashboardproviders.yaml': {
            'apiVersion': 1,
            'providers': [
                {
                    'name': 'default',
                    'orgId': 1,
                    'folder': '',
                    'type': 'file',
                    'disableDeletion': False,
                    'editable': True,
                    'options': {
                        'path': '/var/lib/grafana/dashboards/default'
                    }
                }
            ]
        }
    }
}

helm_resource(
    'grafana',
    'grafana/grafana',
    namespace='observability',
    flags=[
        '--set', 'replicas=1',
        '--set', 'adminUser=admin',
        '--set', 'adminPassword=admin',
        '--set', 'persistence.enabled=false',
        '--set', 'env.GF_AUTH_ANONYMOUS_ENABLED=true',
        '--set', 'env.GF_AUTH_ANONYMOUS_ORG_ROLE=Admin',
        '--set', 'env.GF_AUTH_DISABLE_LOGIN_FORM=true',
        '--set', 'env.GF_SECURITY_ALLOW_EMBEDDING=true',
        '--set-json', 'datasources={}'.format(encode_json(grafana_datasources)),
    ],
    labels=['viz', 'grafana'],
    resource_deps=['tempo', 'mimir', 'loki'],
    port_forwards=['3000:3000']
)

print('✅ Grafana configured (visualization)')

# ==============================================================================
# HOT RELOAD - WATCH JSONNET FILES
# ==============================================================================

# Watch lib/ directory for Jsonnet changes
watch_file('lib/')
watch_file('environments/')

# Tanka apply on changes
local_resource(
    'tanka-dev-sync',
    'tk apply --dangerous-auto-approve environments/dev',
    deps=['lib/', 'environments/dev/'],
    labels=['tanka', 'hot-reload'],
    resource_deps=['grafana', 'alloy', 'obi'],
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL
)

print('✅ Hot reload configured for Jsonnet files')

# ==============================================================================
# HEALTH CHECKS
# ==============================================================================

local_resource(
    'health-check',
    """
    echo "🏥 Health Check Status:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    kubectl get pods -n observability
    echo ""
    echo "🔗 Service Endpoints:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Grafana:  http://localhost:3000"
    echo "Tempo:    http://localhost:3200"
    echo "Mimir:    http://localhost:9009"
    echo "Loki:     http://localhost:3100"
    echo "Alloy:    http://localhost:12345"
    """,
    deps=[],
    labels=['monitoring'],
    resource_deps=['grafana', 'tempo', 'mimir', 'loki', 'alloy', 'obi'],
    auto_init=True,
    trigger_mode=TRIGGER_MODE_MANUAL
)

# ==============================================================================
# LOG AGGREGATION
# ==============================================================================

local_resource(
    'view-logs',
    """
    echo "📋 Component Logs:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Use: kubectl logs -n observability -l app=<component> --tail=50"
    echo ""
    echo "Components: obi, alloy, tempo, mimir, loki, grafana"
    """,
    deps=[],
    labels=['monitoring'],
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL
)

# ==============================================================================
# CLEANUP HELPER
# ==============================================================================

local_resource(
    'cleanup',
    """
    echo "🧹 Cleaning up MOP resources..."
    kubectl delete namespace observability --ignore-not-found=true
    echo "✅ Cleanup complete"
    """,
    deps=[],
    labels=['tools'],
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL
)

# ==============================================================================
# STARTUP BANNER
# ==============================================================================

print("""
╔══════════════════════════════════════════════════════════════════════╗
║                   MOP - Managed Observability Platform               ║
║                        Local Development Environment                 ║
╚══════════════════════════════════════════════════════════════════════╝

📦 Components:
  • OBI (eBPF)      - Zero-code instrumentation
  • Alloy           - Telemetry pipeline
  • Tempo           - Distributed tracing
  • Mimir           - Metrics storage (Prometheus-compatible)
  • Loki            - Log aggregation
  • Grafana         - Visualization

🔗 Service Endpoints:
  • Grafana:  http://localhost:3000 (admin/admin)
  • Tempo:    http://localhost:3200
  • Mimir:    http://localhost:9009/prometheus
  • Loki:     http://localhost:3100
  • Alloy:    http://localhost:12345

🛠️  Manual Resources:
  • tanka-dev-sync  - Apply Tanka changes
  • health-check    - Check component status
  • view-logs       - View aggregated logs
  • cleanup         - Remove all resources

🔥 Hot Reload:
  • Changes to lib/ and environments/ trigger Tanka sync
  • Use 'tanka-dev-sync' resource to manually apply

📚 Documentation:
  • Architecture: docs/architecture/README.md
  • Workstreams:  docs/workstreams/
  • Research:     docs/research/

⚡ Quick Commands:
  • Restart a component:  kubectl rollout restart deployment/<name> -n observability
  • View logs:            kubectl logs -n observability -l app=<name> --tail=50
  • Port forward:         kubectl port-forward -n observability svc/<name> <port>

🚀 Happy developing!
""")

# ==============================================================================
# RESOURCE FILTERING
# ==============================================================================

# Enable selective resource execution
if cfg.get('to-run'):
    enabled_resources = []
    for arg in cfg.get('to-run'):
        if arg in groups:
            enabled_resources += groups[arg]
        else:
            enabled_resources.append(arg)

    config.set_enabled_resources(enabled_resources)
