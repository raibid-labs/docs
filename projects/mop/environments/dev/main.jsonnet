// Development environment for MOP
// Uses minimal resources and local storage

local config = import '../../lib/config.libsonnet';
local k = import 'k.libsonnet';

{
  _config:: config.environments.dev,

  // Create namespace
  namespace: k.core.v1.namespace.new(self._config.namespace) {
    metadata+: {
      labels: config.commonLabels + {
        environment: 'dev',
      },
    },
  },

  // TODO: Add component deployments
  // Components will be added in subsequent workstreams:
  // - OBI DaemonSet (Workstream 2)
  // - Alloy StatefulSet (Workstream 2)
  // - Tempo StatefulSet (Workstream 3)
  // - Mimir StatefulSet (Workstream 3)
  // - Loki StatefulSet (Workstream 3)
  // - Grafana Deployment (Workstream 3)
}
