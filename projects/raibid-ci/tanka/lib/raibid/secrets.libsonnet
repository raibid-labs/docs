// secrets.libsonnet - Secret and ConfigMap management for raibid-ci
// Centralized configuration and secret templates

local k = import '../k.libsonnet';
local config = import './config.libsonnet';

{
  // Create common ConfigMap with application configuration
  // Parameters:
  //   namespace: Kubernetes namespace
  //   name: ConfigMap name (default: 'raibid-config')
  //   redisUrl: Redis connection URL
  //   giteaUrl: Gitea server URL
  //   queueName: Redis Streams queue name
  //   data: Additional configuration data
  configMap(
    namespace,
    name='raibid-config',
    redisUrl='redis://redis-master:6379',
    giteaUrl='http://gitea:3000',
    queueName='raibid:jobs',
    data={}
  ):: {
    apiVersion: 'v1',
    kind: 'ConfigMap',
    metadata: {
      name: name,
      namespace: namespace,
      labels: config.labels.forComponent('config'),
    },
    data: {
      // Redis configuration
      REDIS_URL: redisUrl,
      QUEUE_NAME: queueName,

      // Gitea configuration
      GITEA_URL: giteaUrl,
      GITEA_API_URL: '%s/api/v1' % giteaUrl,

      // Log configuration
      LOG_LEVEL: 'info',
      LOG_FORMAT: 'json',

      // Server configuration
      SERVER_HOST: '0.0.0.0',
      SERVER_PORT: std.toString(config.ports.server.http),

      // Agent configuration
      AGENT_TIMEOUT: '3600',  // 1 hour
      AGENT_MAX_RETRIES: '3',
    } + data,
  },

  // Create Secret for sensitive configuration
  // Parameters:
  //   namespace: Kubernetes namespace
  //   name: Secret name (default: 'raibid-secrets')
  //   redisPassword: Redis password (empty for MVP)
  //   giteaToken: Gitea API token (empty for MVP)
  //   data: Additional secret data
  secret(
    namespace,
    name='raibid-secrets',
    redisPassword='',
    giteaToken='',
    data={}
  ):: {
    apiVersion: 'v1',
    kind: 'Secret',
    metadata: {
      name: name,
      namespace: namespace,
      labels: config.labels.forComponent('secrets'),
    },
    type: 'Opaque',
    stringData: {
      // Redis authentication (disabled for MVP)
      [if redisPassword != '' then 'REDIS_PASSWORD']: redisPassword,

      // Gitea API token (for repository access)
      [if giteaToken != '' then 'GITEA_TOKEN']: giteaToken,
    } + data,
  },

  // Create Docker registry Secret for pulling images
  // Parameters:
  //   namespace: Kubernetes namespace
  //   name: Secret name (default: 'regcred')
  //   server: Registry server
  //   username: Registry username
  //   password: Registry password
  //   email: User email
  dockerConfigSecret(
    namespace,
    name='regcred',
    server='gitea.raibid-system.svc.cluster.local:5000',
    username='',
    password='',
    email='admin@raibid.local'
  ):: {
    apiVersion: 'v1',
    kind: 'Secret',
    metadata: {
      name: name,
      namespace: namespace,
      labels: config.labels.forComponent('registry'),
    },
    type: 'kubernetes.io/dockerconfigjson',
    stringData: {
      '.dockerconfigjson': std.manifestJsonEx({
        auths: {
          [server]: {
            username: username,
            password: password,
            email: email,
            auth: std.base64(username + ':' + password),
          },
        },
      }, '  '),
    },
  },

  // Create TriggerAuthentication for KEDA Redis Streams scaler
  // Parameters:
  //   namespace: Kubernetes namespace
  //   name: TriggerAuthentication name (default: 'redis-trigger-auth')
  //   secretName: Secret containing Redis credentials
  redisTriggerAuth(
    namespace,
    name='redis-trigger-auth',
    secretName='raibid-secrets'
  ):: {
    apiVersion: 'keda.sh/v1alpha1',
    kind: 'TriggerAuthentication',
    metadata: {
      name: name,
      namespace: namespace,
      labels: config.labels.forComponent('keda'),
    },
    spec: {
      secretTargetRef: [
        {
          parameter: 'password',
          name: secretName,
          key: 'REDIS_PASSWORD',
        },
      ],
    },
  },

  // Helper to create all common resources at once
  // Parameters:
  //   namespace: Kubernetes namespace
  //   redisUrl: Redis connection URL
  //   giteaUrl: Gitea server URL
  all(namespace, redisUrl='redis://redis-master:6379', giteaUrl='http://gitea:3000'):: {
    configMap: $.configMap(
      namespace,
      redisUrl=redisUrl,
      giteaUrl=giteaUrl
    ),

    secret: $.secret(namespace),

    // Note: Docker registry secret and trigger auth are optional for MVP
    // Uncomment when needed:
    // dockerConfig: $.dockerConfigSecret(namespace),
    // redisTriggerAuth: $.redisTriggerAuth(namespace),
  },
}
