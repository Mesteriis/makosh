import type { Preview } from '@storybook/vue3-vite'
import { withThemeByDataAttribute } from '@storybook/addon-themes'
import { QueryClient, VUE_QUERY_CLIENT } from '@tanstack/vue-query'
import { provide } from 'vue'
import { initialize, mswLoader } from 'msw-storybook-addon'
import { storybookLocaleToolbarItems } from '../stories/ui/storybook-i18n'
import { isUiThemeName, themeNameToSelection } from '../src/shared/ui/foundation/theme'
import '../src/style.css'
import '../src/styles/surfaces.css'
import '../src/styles/theme-classes.css'
import '../src/shared/ui/styles/index.css'

initialize({
	onUnhandledRequest: 'bypass',
	serviceWorker: {
		url: '/mockServiceWorker.js'
	}
})

const preview: Preview = {
	globalTypes: {
		theme: {
			description: 'Макошь UI theme',
			defaultValue: 'base-light',
			toolbar: {
				title: 'Theme',
				icon: 'circlehollow',
				items: [
					{ value: 'base-light', title: 'Base Light' },
					{ value: 'base-dark', title: 'Base Dark' },
					{ value: 'makosh-light', title: 'Макошь Light' },
					{ value: 'makosh-dark', title: 'Макошь Dark' }
				],
				dynamicTitle: true
			}
		},
		locale: {
			description: 'Макошь UI locale',
			defaultValue: 'ru',
			toolbar: {
				title: 'Locale',
				icon: 'globe',
				items: storybookLocaleToolbarItems,
				dynamicTitle: true
			}
		}
	},
	decorators: [
		withThemeByDataAttribute({
			themes: {
				'base-light': 'base-light',
				'base-dark': 'base-dark',
				'makosh-light': 'makosh-light',
				'makosh-dark': 'makosh-dark'
			},
			defaultTheme: 'base-light',
			attributeName: 'data-ui-theme'
		}),
		(story, context) => ({
			components: { story },
			setup() {
				provide(VUE_QUERY_CLIENT, new QueryClient())
				const theme = isUiThemeName(context.globals.theme) ? context.globals.theme : 'base-light'
				const themeSelection = themeNameToSelection(theme)
				return {
					locale: context.globals.locale,
					storyHeading: [context.title, context.name].filter(Boolean).join(' / '),
					theme,
					themeFamily: themeSelection.family,
					themeMode: themeSelection.mode
				}
			},
			template: '<main :data-ui-theme="theme" :data-ui-theme-family="themeFamily" :data-ui-theme-mode="themeMode" :data-ui-locale="locale" :lang="locale" class="storybook-shell"><h1 class="makosh-sr-only">{{ storyHeading }}</h1><story /></main>'
		})
	],
	loaders: [mswLoader],
	parameters: {
		actions: { argTypesRegex: '^on[A-Z].*' },
		controls: {
			expanded: true,
			matchers: {
				color: /(background|color)$/i,
				date: /Date$/i
			}
		},
		backgrounds: {
			disable: true
		},
		layout: 'fullscreen',
		docs: {
			toc: true
		},
		designToken: {
			defaultTab: 'Colors'
		},
		msw: {
			handlers: []
		},
		viewport: {
			options: {
				uhd4k: {
					name: '4K',
					styles: { width: '3840px', height: '2160px' }
				},
				ipadPro1292020: {
					name: 'iPad Pro 12.9" (2020)',
					styles: { width: '1024px', height: '1366px' }
				},
				zFold7Open: {
					name: 'Samsung Z Fold 7 (open)',
					styles: { width: '1968px', height: '2184px' }
				},
				zFold7Closed: {
					name: 'Samsung Z Fold 7 (closed)',
					styles: { width: '1080px', height: '2520px' }
				},
				macbookPro14: {
					name: 'MacBook Pro 14"',
					styles: { width: '1512px', height: '982px' }
				}
			}
		},
		pseudo: {
			rootSelector: '.storybook-shell'
		}
	}
}

export default preview
