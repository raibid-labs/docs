// util.libsonnet - Utility helpers for raibid-ci
// Provides helpers for environment variables, secrets, and common Kubernetes patterns

local k = import 'k.libsonnet';

{
  // Environment variable helpers
  env:: {
    // Create env var from literal value
    // Usage: util.env.value('LOG_LEVEL', 'info')
    value(name, value):: {
      name: name,
      value: value,
    },

    // Create env var from ConfigMap
    // Usage: util.env.configMap('CONFIG_PATH', 'my-config', 'path')
    configMap(name, configMapName, key):: {
      name: name,
      valueFrom: {
        configMapKeyRef: {
          name: configMapName,
          key: key,
        },
      },
    },

    // Create env var from Secret
    // Usage: util.env.secret('API_KEY', 'my-secret', 'key')
    secret(name, secretName, key, optional=false):: {
      name: name,
      valueFrom: {
        secretKeyRef: {
          name: secretName,
          key: key,
          optional: optional,
        },
      },
    },

    // Create env var from field ref (pod metadata)
    // Usage: util.env.fieldRef('POD_NAME', 'metadata.name')
    fieldRef(name, fieldPath):: {
      name: name,
      valueFrom: {
        fieldRef: {
          fieldPath: fieldPath,
        },
      },
    },

    // Create env var from resource field ref
    // Usage: util.env.resourceRef('MEM_LIMIT', 'limits.memory')
    resourceRef(name, resource):: {
      name: name,
      valueFrom: {
        resourceFieldRef: {
          resource: resource,
        },
      },
    },

    // Import all env vars from ConfigMap
    // Returns envFrom array, not env array
    fromConfigMap(configMapName):: {
      configMapRef: {
        name: configMapName,
      },
    },

    // Import all env vars from Secret
    // Returns envFrom array, not env array
    fromSecret(secretName, optional=false):: {
      secretRef: {
        name: secretName,
        optional: optional,
      },
    },
  },

  // Volume and mount helpers
  volume:: {
    // EmptyDir volume
    emptyDir(name, sizeLimit=null):: {
      name: name,
      emptyDir: if sizeLimit != null then { sizeLimit: sizeLimit } else {},
    },

    // ConfigMap volume
    configMap(name, configMapName, items=null, defaultMode=null):: {
      name: name,
      configMap: {
        name: configMapName,
        [if items != null then 'items']: items,
        [if defaultMode != null then 'defaultMode']: defaultMode,
      },
    },

    // Secret volume
    secret(name, secretName, items=null, defaultMode=null, optional=false):: {
      name: name,
      secret: {
        secretName: secretName,
        [if items != null then 'items']: items,
        [if defaultMode != null then 'defaultMode']: defaultMode,
        optional: optional,
      },
    },

    // PersistentVolumeClaim volume
    pvc(name, claimName):: {
      name: name,
      persistentVolumeClaim: {
        claimName: claimName,
      },
    },

    // HostPath volume (use sparingly, prefer PVCs)
    hostPath(name, path, type='Directory'):: {
      name: name,
      hostPath: {
        path: path,
        type: type,
      },
    },
  },

  // Volume mount helpers
  mount:: {
    // Basic mount
    simple(name, mountPath, readOnly=false):: {
      name: name,
      mountPath: mountPath,
      readOnly: readOnly,
    },

    // Mount with subPath
    subPath(name, mountPath, subPath, readOnly=false):: {
      name: name,
      mountPath: mountPath,
      subPath: subPath,
      readOnly: readOnly,
    },
  },

  // Probe helpers
  probe:: {
    // HTTP GET probe
    http(path, port, initialDelay=10, period=10, timeout=1, successThreshold=1, failureThreshold=3):: {
      httpGet: {
        path: path,
        port: port,
        scheme: 'HTTP',
      },
      initialDelaySeconds: initialDelay,
      periodSeconds: period,
      timeoutSeconds: timeout,
      successThreshold: successThreshold,
      failureThreshold: failureThreshold,
    },

    // TCP socket probe
    tcp(port, initialDelay=10, period=10, timeout=1, successThreshold=1, failureThreshold=3):: {
      tcpSocket: {
        port: port,
      },
      initialDelaySeconds: initialDelay,
      periodSeconds: period,
      timeoutSeconds: timeout,
      successThreshold: successThreshold,
      failureThreshold: failureThreshold,
    },

    // Exec command probe
    exec(command, initialDelay=10, period=10, timeout=1, successThreshold=1, failureThreshold=3):: {
      exec: {
        command: command,
      },
      initialDelaySeconds: initialDelay,
      periodSeconds: period,
      timeoutSeconds: timeout,
      successThreshold: successThreshold,
      failureThreshold: failureThreshold,
    },
  },

  // Container helpers
  container:: {
    // Create init container
    init(name, image, command=null, args=null, env=null, volumeMounts=null):: {
      name: name,
      image: image,
      [if command != null then 'command']: command,
      [if args != null then 'args']: args,
      [if env != null then 'env']: env,
      [if volumeMounts != null then 'volumeMounts']: volumeMounts,
    },

    // Create sidecar container
    sidecar(name, image, ports=null, env=null, volumeMounts=null, resources=null):: {
      name: name,
      image: image,
      [if ports != null then 'ports']: ports,
      [if env != null then 'env']: env,
      [if volumeMounts != null then 'volumeMounts']: volumeMounts,
      [if resources != null then 'resources']: resources,
    },
  },

  // Security context helpers
  security:: {
    // Non-root user security context
    nonRoot(uid=1000, gid=1000, fsGroup=1000):: {
      runAsNonRoot: true,
      runAsUser: uid,
      runAsGroup: gid,
      fsGroup: fsGroup,
      seccompProfile: {
        type: 'RuntimeDefault',
      },
    },

    // Container security context
    container(allowPrivilegeEscalation=false, readOnlyRootFilesystem=true, runAsNonRoot=true):: {
      allowPrivilegeEscalation: allowPrivilegeEscalation,
      readOnlyRootFilesystem: readOnlyRootFilesystem,
      runAsNonRoot: runAsNonRoot,
      capabilities: {
        drop: ['ALL'],
      },
      seccompProfile: {
        type: 'RuntimeDefault',
      },
    },
  },

  // Merge helpers
  merge:: {
    // Deep merge two objects
    // Later object takes precedence
    objects(a, b):: a + b,

    // Merge arrays (concatenate)
    arrays(a, b):: a + b,
  },

  // Conditional helpers
  when:: {
    // Only include field if condition is true
    // Usage: when.true(enabled, 'field', value)
    true(condition, field, value):: if condition then { [field]: value } else {},

    // Only include field if value is not null
    // Usage: when.notNull('field', maybeValue)
    notNull(field, value):: if value != null then { [field]: value } else {},
  },
}
