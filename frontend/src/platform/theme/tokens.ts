/** Typed Макошь design token constants for use outside Tailwind classes */
export const theme = {
	font: {
		sans: [
			'Inter',
			'SF Pro Display',
			'ui-sans-serif',
			'system-ui',
			'-apple-system',
			'BlinkMacSystemFont',
			'Segoe UI',
			'sans-serif'
		].join(', ')
	},
	color: {
		bg: '#02090b',
		bgRaised: '#020d10',
		surface: '#06181b',
		surfaceDeep: '#041215',
		text: '#eefefb',
		textStrong: '#f2fffd',
		textBright: '#ffffff',
		textSoft: '#dcefed',
		textMuted: '#91a8a8',
		textSubtle: '#8ea4a6',
		textDim: '#849ca0',
		accent: '#2df0ce',
		accentStrong: '#25d8bd',
		accentSoft: '#9ee8df',
		accentContrast: '#032522',
		danger: '#ffabab',
		dangerStrong: '#ef3140'
	},
	radius: {
		xs: '4px',
		sm: '6px',
		control: '7px',
		md: '8px',
		lg: '14px',
		xl: '18px',
		pill: '999px',
		round: '50%'
	},
	space: {
		'1': '4px',
		'2': '8px',
		'3': '12px',
		'4': '16px',
		'5': '20px',
		'6': '24px'
	},
	layout: {
		row: '37px',
		gap: '10px',
		columns: 12
	}
} as const
