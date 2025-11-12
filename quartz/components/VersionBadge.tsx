import { QuartzComponentConstructor, QuartzComponentProps } from "./types"
import { classNames } from "../util/lang"
import style from "./styles/versionBadge.scss"

interface VersionBadgeOptions {
  /**
   * Whether to display the badge
   */
  showBadge: boolean
}

const defaultOptions: VersionBadgeOptions = {
  showBadge: true,
}

export default ((opts?: Partial<VersionBadgeOptions>) => {
  const options: VersionBadgeOptions = { ...defaultOptions, ...opts }

  function VersionBadge({ fileData, displayClass }: QuartzComponentProps) {
    // Only show on project index pages
    if (!fileData.frontmatter?.tags?.includes("project") || !options.showBadge) {
      return null
    }

    // Extract version info from frontmatter if available
    const version = fileData.frontmatter?.version
    const releaseUrl = fileData.frontmatter?.releaseUrl
    const lastUpdated = fileData.frontmatter?.lastUpdated

    if (!version && !lastUpdated) {
      return null
    }

    return (
      <div class={classNames(displayClass, "version-badge")}>
        {version && (
          <span class="version-tag">
            {releaseUrl ? (
              <a href={releaseUrl} target="_blank" rel="noopener noreferrer">
                📦 {version}
              </a>
            ) : (
              <span>📦 {version}</span>
            )}
          </span>
        )}
        {lastUpdated && (
          <span class="last-updated">
            🕐 Updated: {lastUpdated}
          </span>
        )}
      </div>
    )
  }

  VersionBadge.css = style

  return VersionBadge
}) satisfies QuartzComponentConstructor
