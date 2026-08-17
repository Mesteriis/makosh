import {
	completeGmailOAuthBrowserCallbackV1,
	mountGmailOAuthCallbackPageV1,
} from './integrations/mail/oauth/gmailOAuthBrowserFlow'
import { completeGmailOAuthSameTabCallbackV1 } from './integrations/mail/oauth/gmailOAuthRedirectFlow'
import './integrations/mail/oauth/gmailOAuthCallbackPage.css'

async function bootstrap(): Promise<void> {
	const sameTabCallback = await completeGmailOAuthSameTabCallbackV1()
	if (sameTabCallback !== 'not_callback') {
		mountGmailOAuthCallbackPageV1(sameTabCallback)
		return
	}
	const popupCallback = completeGmailOAuthBrowserCallbackV1()
	if (popupCallback !== 'not_callback') {
		mountGmailOAuthCallbackPageV1(popupCallback)
		return
	}
	const { mountClientApp } = await import('./app/bootstrap')
	mountClientApp()
}

void bootstrap()
