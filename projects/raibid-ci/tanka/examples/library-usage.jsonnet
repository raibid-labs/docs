// library-usage.jsonnet - Examples of using raibid-ci libraries
// This file demonstrates common patterns and helper usage

local k8s = import '../lib/k8s.libsonnet';
local config = import '../lib/raibid/config.libsonnet';
local util = import '../lib/raibid/util.libsonnet';

// Example 1: Creating a Deployment with standard configuration
local exampleDeployment =
  k8s.deployment.new(
    name='example-service',
    component='backend',
    containers=[
      k8s.core.container.new('app', 'nginx:1.25')
      + k8s.core.container.withPorts([
        k8s.containerPort('http', 8080),
        k8s.containerPort('metrics', 9090),
      ])
      + k8s.core.container.withEnv([
        util.env.value('LOG_LEVEL', 'info'),
        util.env.fieldRef('POD_NAME', 'metadata.name'),
        util.env.fieldRef('POD_NAMESPACE', 'metadata.namespace'),
        util.env.secret('API_KEY', 'app-secrets', 'api-key'),
        util.env.configMap('CONFIG_PATH', 'app-config', 'config.yaml'),
      ])
      + k8s.core.container.withVolumeMounts([
        util.mount.simple('config', '/etc/config', readOnly=true),
        util.mount.simple('cache', '/var/cache'),
      ])
      + k8s.core.container.withResources(config.resources.medium)
      + k8s.core.container.livenessProbe.mixin.httpGet.withPath('/health')
      + k8s.core.container.livenessProbe.mixin.httpGet.withPort('http')
      + k8s.core.container.readinessProbe.mixin.httpGet.withPath('/ready')
      + k8s.core.container.readinessProbe.mixin.httpGet.withPort('http'),
    ],
    replicas=3,
    namespace=config.namespace
  )
  + k8s.deployment.spec.template.spec.withVolumes([
    util.volume.configMap('config', 'app-config'),
    util.volume.emptyDir('cache', '1Gi'),
  ])
  + k8s.deployment.spec.template.spec.withSecurityContext(
    util.security.nonRoot(uid=1000, gid=1000, fsGroup=1000)
  );

// Example 2: Creating a Service
local exampleService =
  k8s.service.clusterIP(
    name='example-service',
    component='backend',
    ports=[
      k8s.servicePort('http', 80, 8080),
      k8s.servicePort('metrics', 9090, 9090),
    ],
    namespace=config.namespace,
    labels=config.annotations.prometheus(port='9090')
  );

// Example 3: Creating a ConfigMap
local exampleConfigMap =
  k8s.configMap.fromData(
    name='app-config',
    data={
      'config.yaml': |||
        server:
          port: 8080
          host: 0.0.0.0
        logging:
          level: info
          format: json
      |||,
      'app.properties': 'key=value\nfoo=bar',
    },
    namespace=config.namespace
  );

// Example 4: Creating a Secret
local exampleSecret =
  k8s.secret.opaque(
    name='app-secrets',
    data={
      'api-key': std.base64('super-secret-key'),
      'db-password': std.base64('database-password'),
    },
    namespace=config.namespace
  );

// Example 5: Creating a ServiceAccount with RBAC
local exampleServiceAccount =
  k8s.serviceAccount.new(
    name='example-sa',
    namespace=config.namespace
  );

local exampleRole =
  k8s.rbacHelpers.role(
    name='example-role',
    rules=[
      {
        apiGroups: [''],
        resources: ['configmaps', 'secrets'],
        verbs: ['get', 'list', 'watch'],
      },
      {
        apiGroups: ['apps'],
        resources: ['deployments'],
        verbs: ['get', 'list'],
      },
    ],
    namespace=config.namespace
  );

local exampleRoleBinding =
  k8s.rbacHelpers.roleBinding(
    name='example-binding',
    roleName='example-role',
    subjects=[
      {
        kind: 'ServiceAccount',
        name: 'example-sa',
        namespace: config.namespace,
      },
    ],
    namespace=config.namespace
  );

// Example 6: Creating a StatefulSet
local exampleStatefulSet =
  k8s.statefulSet.new(
    name='example-stateful',
    component='database',
    serviceName='example-stateful-headless',
    containers=[
      k8s.core.container.new('db', 'postgres:15')
      + k8s.core.container.withPorts([
        k8s.containerPort('postgres', 5432),
      ])
      + k8s.core.container.withEnvFrom([
        util.env.fromSecret('postgres-env'),
      ])
      + k8s.core.container.withVolumeMounts([
        util.mount.simple('data', '/var/lib/postgresql/data'),
      ])
      + k8s.core.container.withResources(config.resources.large),
    ],
    replicas=3,
    namespace=config.namespace
  )
  + k8s.statefulSet.withVolumeClaimTemplate(
    k8s.pvc.new(
      name='data',
      storageClass='local-path',
      size='10Gi',
      accessModes=['ReadWriteOnce']
    )
  );

// Example 7: Using labels and naming conventions
local componentLabels = config.labels.forComponent('api-server');
local instanceLabels = config.labels.forInstance('api-server', 'primary');
local selectorLabels = config.labels.selector('api-server');

local deploymentName = config.naming.resource('api-server', 'deployment');
local serviceName = config.naming.service('api-server');
local configName = config.naming.config('api-server');
local secretName = config.naming.secret('api-server');

// Example 8: Environment-specific configuration
local localEnv = config.environments['local'];
local registry = localEnv.registry;
local domain = localEnv.domain;

// Example 9: Container with init container
local deploymentWithInit =
  exampleDeployment
  + k8s.deployment.withInitContainer(
    util.container.init(
      name='migration',
      image='myapp:latest',
      command=['./migrate'],
      env=[
        util.env.secret('DB_URL', 'db-secret', 'url'),
      ]
    )
  );

// Example 10: Job with TTL
local exampleJob =
  k8s.job.new(
    name='data-migration',
    component='jobs',
    containers=[
      k8s.core.container.new('migrator', 'myapp:latest')
      + k8s.core.container.withCommand(['./migrate', 'up']),
    ],
    namespace=config.namespace
  )
  + k8s.job.withBackoffLimit(3)
  + k8s.job.withTtlAfterFinished(3600);

// Export all examples
{
  // Core resources
  deployment: exampleDeployment,
  service: exampleService,
  configMap: exampleConfigMap,
  secret: exampleSecret,

  // RBAC
  serviceAccount: exampleServiceAccount,
  role: exampleRole,
  roleBinding: exampleRoleBinding,

  // Stateful workloads
  statefulSet: exampleStatefulSet,

  // Jobs
  job: exampleJob,

  // Advanced patterns
  deploymentWithInit: deploymentWithInit,

  // Configuration examples (not actual resources)
  _config: {
    labels: {
      component: componentLabels,
      instance: instanceLabels,
      selector: selectorLabels,
    },
    naming: {
      deployment: deploymentName,
      service: serviceName,
      config: configName,
      secret: secretName,
    },
    environment: localEnv,
  },
}
