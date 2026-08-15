import {
	completeGmailOAuthBrowserCallbackV1,
	mountGmailOAuthCallbackPageV1,
} from './integrations/mail/oauth/gmailOAuthBrowserFlow'
import './integrations/mail/oauth/gmailOAuthCallbackPage.css'

const gmailOAuthCallback = completeGmailOAuthBrowserCallbackV1()
if (gmailOAuthCallback !== 'not_callback') {
	mountGmailOAuthCallbackPageV1(gmailOAuthCallback)
} else {
	void import('./app/bootstrap').then(({ mountClientApp }) => mountClientApp())
}
