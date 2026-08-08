/**
 * Fixed local asset inventory used by the compiled Макошь UI kit.
 * Gateway may serve only these bundled paths; clients never accept asset URLs
 * from bootstrap or another remote control-plane response.
 */
export const makoshBrandAssets = {
	logoMarkDark: '/assets/makosh-logo-mark-dark.png',
	logoMarkLight: '/assets/makosh-logo-mark-light.png'
} as const

export const makoshShellBackgroundAssetPaths = [
	'/assets/shell-backgrounds/network-mesh.webp',
	'/assets/shell-backgrounds/data-stream.webp',
	'/assets/shell-backgrounds/node-frame.webp',
	'/assets/shell-backgrounds/eclipse-grid.webp',
	'/assets/shell-backgrounds/dna-blueprint.webp',
	'/assets/shell-backgrounds/forest-network.webp',
	'/assets/shell-backgrounds/forest-stream.webp',
	'/assets/shell-backgrounds/knowledge-map.webp',
	'/assets/shell-backgrounds/rune-gold.webp',
	'/assets/shell-backgrounds/rune-teal.webp'
] as const
