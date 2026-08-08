set -eu

cd /home/zulip/deployments/current

su zulip -c './manage.py shell' <<'PY'
import json
import os

from zerver.actions.create_realm import do_create_realm
from zerver.actions.create_user import do_create_user
from zerver.actions.streams import bulk_add_subscriptions
from zerver.lib.streams import create_stream_if_needed
from zerver.models import Realm, UserProfile

realm_name = os.environ["MAKOSH_REALM_NAME"]
stream_name = os.environ["MAKOSH_STREAM_NAME"]
owner_email = os.environ["MAKOSH_OWNER_EMAIL"]
owner_name = os.environ["MAKOSH_OWNER_NAME"]
owner_password = os.environ["MAKOSH_OWNER_PASSWORD"]
human_email = os.environ["MAKOSH_HUMAN_EMAIL"]
human_name = os.environ["MAKOSH_HUMAN_NAME"]
bot_email = os.environ["MAKOSH_BOT_EMAIL"]
bot_name = os.environ["MAKOSH_BOT_NAME"]

realm = Realm.objects.filter(string_id="").first()
if realm is None:
    realm = do_create_realm(string_id="", name=realm_name)

owner = UserProfile.objects.filter(realm=realm, delivery_email__iexact=owner_email).first()
if owner is None:
    owner = do_create_user(
        owner_email,
        owner_password,
        realm,
        owner_name,
        role=UserProfile.ROLE_REALM_OWNER,
        realm_creation=True,
        acting_user=None,
    )

human = UserProfile.objects.filter(realm=realm, delivery_email__iexact=human_email).first()
if human is None:
    human = do_create_user(
        human_email,
        None,
        realm,
        human_name,
        acting_user=owner,
    )

bot = UserProfile.objects.filter(realm=realm, delivery_email__iexact=bot_email).first()
if bot is None:
    bot = do_create_user(
        bot_email,
        None,
        realm,
        bot_name,
        bot_type=UserProfile.DEFAULT_BOT,
        bot_owner=owner,
        acting_user=owner,
    )

stream, _ = create_stream_if_needed(realm, stream_name, acting_user=owner)
bulk_add_subscriptions(realm, [stream], [owner, human, bot], acting_user=owner)

print("MAKOSH_ZULIP_PROVISION " + json.dumps({
    "owner_email": owner.delivery_email,
    "owner_user_id": owner.id,
    "owner_api_key": owner.api_key,
    "human_email": human.delivery_email,
    "human_user_id": human.id,
    "human_api_key": human.api_key,
    "bot_email": bot.delivery_email,
    "bot_user_id": bot.id,
    "bot_api_key": bot.api_key,
    "stream_name": stream.name,
}, sort_keys=True))
PY
