// k8s.libsonnet - Kubernetes API shortcuts and helpers
// Provides convenient wrappers around k8s-libsonnet for common patterns

local k = import 'k.libsonnet';
local config = import 'raibid/config.libsonnet';

{
  // Core API shortcuts
  core:: k.core.v1,
  apps:: k.apps.v1,
  batch:: k.batch.v1,
  rbac:: k.rbac.v1,
  policy:: k.policy.v1,
  networking:: k.networking.v1,

  // Namespace helper
  namespace(name, labels={}, annotations={})::
    $.core.namespace.new(name)
    + $.core.namespace.metadata.withLabels(config.labels.common + labels)
    + $.core.namespace.metadata.withAnnotations(config.annotations.common + annotations),

  // ConfigMap helpers
  configMap:: {
    // Create ConfigMap from data
    fromData(name, data, namespace=config.namespace, labels={})::
      $.core.configMap.new(name, data)
      + $.core.configMap.metadata.withNamespace(namespace)
      + $.core.configMap.metadata.withLabels(config.labels.common + labels),

    // Create ConfigMap from file-like data
    fromFiles(name, files, namespace=config.namespace, labels={})::
      $.core.configMap.new(name)
      + $.core.configMap.withData(files)
      + $.core.configMap.metadata.withNamespace(namespace)
      + $.core.configMap.metadata.withLabels(config.labels.common + labels),
  },

  // Secret helpers
  secret:: {
    // Create opaque secret
    opaque(name, data, namespace=config.namespace, labels={})::
      $.core.secret.new(name, data, type='Opaque')
      + $.core.secret.metadata.withNamespace(namespace)
      + $.core.secret.metadata.withLabels(config.labels.common + labels),

    // Create TLS secret
    tls(name, cert, key, namespace=config.namespace, labels={})::
      $.core.secret.new(name, {}, type='kubernetes.io/tls')
      + $.core.secret.withStringData({
        'tls.crt': cert,
        'tls.key': key,
      })
      + $.core.secret.metadata.withNamespace(namespace)
      + $.core.secret.metadata.withLabels(config.labels.common + labels),

    // Create docker registry secret
    dockerRegistry(name, server, username, password, email, namespace=config.namespace, labels={})::
      $.core.secret.new(name, {}, type='kubernetes.io/dockerconfigjson')
      + $.core.secret.withStringData({
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
      })
      + $.core.secret.metadata.withNamespace(namespace)
      + $.core.secret.metadata.withLabels(config.labels.common + labels),
  },

  // Service helpers
  service:: {
    // Create ClusterIP service
    clusterIP(name, component, ports, namespace=config.namespace, labels={})::
      $.core.service.new(
        name,
        config.labels.selector(component),
        ports
      )
      + $.core.service.metadata.withNamespace(namespace)
      + $.core.service.metadata.withLabels(config.labels.forComponent(component) + labels)
      + $.core.service.spec.withType('ClusterIP'),

    // Create headless service
    headless(name, component, ports, namespace=config.namespace, labels={})::
      $.service.clusterIP(name, component, ports, namespace, labels)
      + $.core.service.spec.withClusterIp('None'),

    // Create NodePort service
    nodePort(name, component, ports, namespace=config.namespace, labels={})::
      $.service.clusterIP(name, component, ports, namespace, labels)
      + $.core.service.spec.withType('NodePort'),

    // Create LoadBalancer service
    loadBalancer(name, component, ports, namespace=config.namespace, labels={})::
      $.service.clusterIP(name, component, ports, namespace, labels)
      + $.core.service.spec.withType('LoadBalancer'),
  },

  // Deployment helpers
  deployment:: {
    // Create basic deployment
    new(name, component, containers, replicas=1, namespace=config.namespace, labels={})::
      $.apps.deployment.new(
        name,
        replicas,
        containers,
        config.labels.selector(component)
      )
      + $.apps.deployment.metadata.withNamespace(namespace)
      + $.apps.deployment.metadata.withLabels(config.labels.forComponent(component) + labels)
      + $.apps.deployment.spec.template.metadata.withLabels(config.labels.forComponent(component) + labels),

    // Add volume to deployment
    withVolume(deployment, volume)::
      deployment
      + $.apps.deployment.spec.template.spec.volumes([volume]),

    // Add init container
    withInitContainer(deployment, container)::
      deployment
      + $.apps.deployment.spec.template.spec.initContainers([container]),

    // Set security context
    withSecurityContext(deployment, securityContext)::
      deployment
      + $.apps.deployment.spec.template.spec.withSecurityContext(securityContext),

    // Add node selector
    withNodeSelector(deployment, nodeSelector)::
      deployment
      + $.apps.deployment.spec.template.spec.withNodeSelector(nodeSelector),

    // Add tolerations
    withTolerations(deployment, tolerations)::
      deployment
      + $.apps.deployment.spec.template.spec.withTolerations(tolerations),

    // Add affinity
    withAffinity(deployment, affinity)::
      deployment
      + $.apps.deployment.spec.template.spec.withAffinity(affinity),
  },

  // StatefulSet helpers
  statefulSet:: {
    // Create basic statefulset
    new(name, component, serviceName, containers, replicas=1, namespace=config.namespace, labels={})::
      $.apps.statefulSet.new(
        name,
        replicas,
        containers,
        config.labels.selector(component),
        volumeClaims=[]
      )
      + $.apps.statefulSet.metadata.withNamespace(namespace)
      + $.apps.statefulSet.metadata.withLabels(config.labels.forComponent(component) + labels)
      + $.apps.statefulSet.spec.withServiceName(serviceName)
      + $.apps.statefulSet.spec.template.metadata.withLabels(config.labels.forComponent(component) + labels),

    // Add volume claim template
    withVolumeClaimTemplate(statefulSet, pvcTemplate)::
      statefulSet
      + $.apps.statefulSet.spec.volumeClaimTemplates([pvcTemplate]),
  },

  // Job helpers
  job:: {
    // Create basic job
    new(name, component, containers, namespace=config.namespace, labels={})::
      $.batch.job.new(name)
      + $.batch.job.metadata.withNamespace(namespace)
      + $.batch.job.metadata.withLabels(config.labels.forComponent(component) + labels)
      + $.batch.job.spec.template.metadata.withLabels(config.labels.forComponent(component) + labels)
      + $.batch.job.spec.template.spec.withContainers(containers)
      + $.batch.job.spec.template.spec.withRestartPolicy('OnFailure'),

    // Set backoff limit
    withBackoffLimit(job, limit)::
      job
      + $.batch.job.spec.withBackoffLimit(limit),

    // Set completion mode
    withCompletions(job, completions, parallelism=1)::
      job
      + $.batch.job.spec.withCompletions(completions)
      + $.batch.job.spec.withParallelism(parallelism),

    // Set TTL after finished
    withTtlAfterFinished(job, seconds)::
      job
      + $.batch.job.spec.withTtlSecondsAfterFinished(seconds),
  },

  // CronJob helpers
  cronJob:: {
    // Create basic cronjob
    new(name, component, schedule, containers, namespace=config.namespace, labels={})::
      $.batch.cronJob.new(name)
      + $.batch.cronJob.metadata.withNamespace(namespace)
      + $.batch.cronJob.metadata.withLabels(config.labels.forComponent(component) + labels)
      + $.batch.cronJob.spec.withSchedule(schedule)
      + $.batch.cronJob.spec.jobTemplate.spec.template.metadata.withLabels(config.labels.forComponent(component) + labels)
      + $.batch.cronJob.spec.jobTemplate.spec.template.spec.withContainers(containers)
      + $.batch.cronJob.spec.jobTemplate.spec.template.spec.withRestartPolicy('OnFailure'),

    // Set concurrency policy
    withConcurrencyPolicy(cronJob, policy)::
      cronJob
      + $.batch.cronJob.spec.withConcurrencyPolicy(policy),

    // Set success/failure history limits
    withHistoryLimits(cronJob, successfulJobsHistoryLimit=3, failedJobsHistoryLimit=1)::
      cronJob
      + $.batch.cronJob.spec.withSuccessfulJobsHistoryLimit(successfulJobsHistoryLimit)
      + $.batch.cronJob.spec.withFailedJobsHistoryLimit(failedJobsHistoryLimit),
  },

  // ServiceAccount helpers
  serviceAccount:: {
    // Create service account
    new(name, namespace=config.namespace, labels={})::
      $.core.serviceAccount.new(name)
      + $.core.serviceAccount.metadata.withNamespace(namespace)
      + $.core.serviceAccount.metadata.withLabels(config.labels.common + labels),

    // Attach to pod spec
    attachToPod(podSpec, serviceAccountName)::
      podSpec
      + { spec+: { serviceAccountName: serviceAccountName } },
  },

  // RBAC helpers
  rbacHelpers:: {
    // Create role
    role(name, rules, namespace=config.namespace, labels={})::
      $.rbac.role.new(name)
      + $.rbac.role.metadata.withNamespace(namespace)
      + $.rbac.role.metadata.withLabels(config.labels.common + labels)
      + $.rbac.role.withRules(rules),

    // Create cluster role
    clusterRole(name, rules, labels={})::
      $.rbac.clusterRole.new(name)
      + $.rbac.clusterRole.metadata.withLabels(config.labels.common + labels)
      + $.rbac.clusterRole.withRules(rules),

    // Create role binding
    roleBinding(name, roleName, subjects, namespace=config.namespace, labels={})::
      $.rbac.roleBinding.new(name)
      + $.rbac.roleBinding.metadata.withNamespace(namespace)
      + $.rbac.roleBinding.metadata.withLabels(config.labels.common + labels)
      + $.rbac.roleBinding.roleRef.withApiGroup('rbac.authorization.k8s.io')
      + $.rbac.roleBinding.roleRef.withKind('Role')
      + $.rbac.roleBinding.roleRef.withName(roleName)
      + $.rbac.roleBinding.withSubjects(subjects),

    // Create cluster role binding
    clusterRoleBinding(name, clusterRoleName, subjects, labels={})::
      $.rbac.clusterRoleBinding.new(name)
      + $.rbac.clusterRoleBinding.metadata.withLabels(config.labels.common + labels)
      + $.rbac.clusterRoleBinding.roleRef.withApiGroup('rbac.authorization.k8s.io')
      + $.rbac.clusterRoleBinding.roleRef.withKind('ClusterRole')
      + $.rbac.clusterRoleBinding.roleRef.withName(clusterRoleName)
      + $.rbac.clusterRoleBinding.withSubjects(subjects),
  },

  // PersistentVolumeClaim helpers
  pvc:: {
    // Create PVC
    new(name, storageClass, size, accessModes=['ReadWriteOnce'], namespace=config.namespace, labels={})::
      $.core.persistentVolumeClaim.new(name)
      + $.core.persistentVolumeClaim.metadata.withNamespace(namespace)
      + $.core.persistentVolumeClaim.metadata.withLabels(config.labels.common + labels)
      + $.core.persistentVolumeClaim.spec.withAccessModes(accessModes)
      + $.core.persistentVolumeClaim.spec.withStorageClassName(storageClass)
      + $.core.persistentVolumeClaim.spec.resources.withRequests({ storage: size }),
  },

  // Container port helper
  containerPort(name, containerPort, protocol='TCP')::
    $.core.containerPort.new(name, containerPort)
    + $.core.containerPort.withProtocol(protocol),

  // Service port helper
  servicePort(name, port, targetPort, protocol='TCP')::
    $.core.servicePort.new(port, targetPort)
    + $.core.servicePort.withName(name)
    + $.core.servicePort.withProtocol(protocol),
}
