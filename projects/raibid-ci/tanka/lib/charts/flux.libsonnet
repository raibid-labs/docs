// flux.libsonnet - Flux Helm chart wrapper for Tanka
// Wraps the Flux chart for GitOps workflows in raibid-ci
// Chart: https://github.com/fluxcd-community/helm-charts

local tanka = import 'github.com/grafana/jsonnet-libs/tanka-util/main.libsonnet';
local helm = tanka.helm.new(std.thisFile);
local config = import '../raibid/config.libsonnet';

{
  // Create Flux deployment with GitOps controllers
  // Parameters:
  //   namespace: Kubernetes namespace
  //   name: Release name (default: 'flux')
  //   values: Additional Helm values to merge
  new(namespace, name='flux', values={}):: {
    local defaultValues = {
      // Source controller - handles Git repositories
      sourceController: {
        create: true,
      },

      // Kustomize controller - applies Kustomizations
      kustomizeController: {
        create: true,
      },

      // Helm controller - manages Helm releases
      helmController: {
        create: true,
      },

      // Notification controller (disabled for MVP)
      notificationController: {
        create: false,
      },

      // Image automation controller (disabled for MVP)
      imageAutomationController: {
        create: false,
      },

      // Image reflector controller (disabled for MVP)
      imageReflectorController: {
        create: false,
      },

      // Network policies (disabled for MVP)
      policies: {
        create: false,
      },

      // RBAC
      rbac: {
        create: true,
      },
    },

    // Merge default values with user-provided values
    local mergedValues = defaultValues + values,

    // Render the Helm chart
    local chart = helm.template(name, '../../vendor/flux2', {
      namespace: namespace,
      values: mergedValues,
      kubeVersion: '1.29',
      includeCrds: true,
    });

    // Return rendered resources
    chart
  },

  // Helper functions for Flux CRDs
  crds:: {
    // Create a GitRepository resource
    // Parameters:
    //   name: GitRepository name
    //   namespace: Kubernetes namespace
    //   url: Git repository URL
    //   branch: Git branch (default: 'main')
    //   interval: Sync interval (default: '1m0s')
    //   secretRef: Optional secret reference for authentication
    gitRepository(name, namespace, url, branch='main', interval='1m0s', secretRef=null):: {
      apiVersion: 'source.toolkit.fluxcd.io/v1',
      kind: 'GitRepository',
      metadata: {
        name: name,
        namespace: namespace,
        labels: config.labels.forComponent('flux'),
      },
      spec: {
        interval: interval,
        url: url,
        ref: {
          branch: branch,
        },
        [if secretRef != null then 'secretRef']: {
          name: secretRef,
        },
      },
    },

    // Create a Kustomization resource
    // Parameters:
    //   name: Kustomization name
    //   namespace: Kubernetes namespace
    //   sourceRef: Source reference (GitRepository)
    //   path: Path in repository (default: './')
    //   prune: Enable pruning (default: true)
    //   interval: Sync interval (default: '5m0s')
    kustomization(name, namespace, sourceRef, path='./', prune=true, interval='5m0s'):: {
      apiVersion: 'kustomize.toolkit.fluxcd.io/v1',
      kind: 'Kustomization',
      metadata: {
        name: name,
        namespace: namespace,
        labels: config.labels.forComponent('flux'),
      },
      spec: {
        interval: interval,
        path: path,
        prune: prune,
        sourceRef: sourceRef,
      },
    },

    // Create a HelmRepository resource
    // Parameters:
    //   name: HelmRepository name
    //   namespace: Kubernetes namespace
    //   url: Helm repository URL
    //   interval: Sync interval (default: '10m0s')
    helmRepository(name, namespace, url, interval='10m0s'):: {
      apiVersion: 'source.toolkit.fluxcd.io/v1beta2',
      kind: 'HelmRepository',
      metadata: {
        name: name,
        namespace: namespace,
        labels: config.labels.forComponent('flux'),
      },
      spec: {
        interval: interval,
        url: url,
      },
    },

    // Create a HelmRelease resource
    // Parameters:
    //   name: HelmRelease name
    //   namespace: Kubernetes namespace
    //   chart: Chart name
    //   sourceRef: HelmRepository reference
    //   values: Helm values
    //   interval: Sync interval (default: '5m0s')
    helmRelease(name, namespace, chart, sourceRef, values={}, interval='5m0s'):: {
      apiVersion: 'helm.toolkit.fluxcd.io/v2beta1',
      kind: 'HelmRelease',
      metadata: {
        name: name,
        namespace: namespace,
        labels: config.labels.forComponent('flux'),
      },
      spec: {
        interval: interval,
        chart: {
          spec: {
            chart: chart,
            sourceRef: sourceRef,
          },
        },
        values: values,
      },
    },
  },
}
