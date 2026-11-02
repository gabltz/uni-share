#!/usr/bin/env bash
# Build and sign static/shortcut/UNI-SHARE.shortcut from PUBLIC_BASE_URL in .env
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ -f .env ]]; then
  # shellcheck disable=SC1091
  set -a
  # shellcheck disable=SC1091
  source .env
  set +a
fi

PUBLIC_BASE_URL="${PUBLIC_BASE_URL:-http://127.0.0.1:8080}"
PUBLIC_BASE_URL="${PUBLIC_BASE_URL%/}"
ENDPOINT="${PUBLIC_BASE_URL}/api/v1/convert"

python3 - "$ENDPOINT" "$ROOT/static/shortcut/UNI-SHARE-unsigned.shortcut" <<'PY'
import plistlib, uuid, pathlib, sys

endpoint = sys.argv[1]
out = pathlib.Path(sys.argv[2])

def U():
    return str(uuid.uuid4()).upper()

api_text_uuid = U()
clipboard_in_uuid = U()
ask_uuid = U()
menu_uuid = U()
menu_group = U()
download_uuid = U()
dict_from_uuid = U()
getvalue_uuid = U()
result_clipboard_uuid = U()
open_uuid = U()
open_menu_group = U()
open_menu_uuid = U()
OCR = "\ufffc"

def text_token_string(text, attachments=None):
    return {
        "Value": {"string": text, "attachmentsByRange": attachments or {}},
        "WFSerializationType": "WFTextTokenString",
    }

def attachment_ref(output_uuid, output_name, typ="ActionOutput"):
    return {"OutputUUID": output_uuid, "OutputName": output_name, "Type": typ}

def aggrandized_attachment(output_uuid, output_name, coercion="WFURLContentItem"):
    """Force URL/text content items into something JSON can serialize."""
    return {
        "OutputUUID": output_uuid,
        "OutputName": output_name,
        "Type": "ActionOutput",
        "Aggrandizements": [
            {
                "Type": "WFCoercionVariableAggrandizement",
                "CoercionItemClass": coercion,
            }
        ],
    }

shortcut_input_att = {"Type": "ExtensionInput", "VariableName": "Shortcut Input"}
actions = []

# 1) Endpoint API (éditable)
actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.gettext",
    "WFWorkflowActionParameters": {
        "UUID": api_text_uuid,
        "CustomOutputName": "URL API de conversion",
        "WFTextActionText": endpoint,
    },
})

# 2) Presse-papiers (fallback si pas de partage)
actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.getclipboard",
    "WFWorkflowActionParameters": {
        "UUID": clipboard_in_uuid,
        "CustomOutputName": "Presse-papiers",
    },
})

# 3) Demander / confirmer le lien (prérempli Share Sheet ou presse-papiers)
#    Default = Shortcut Input ; si vide au runtime iOS utilise souvent le presse-papiers
#    via NoInputBehavior + cette étape Ask évite le body JSON vide.
actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.ask",
    "WFWorkflowActionParameters": {
        "UUID": ask_uuid,
        "CustomOutputName": "Lien du morceau",
        "WFAskActionPrompt": "Lien du morceau (Share, presse-papiers, ou colle ici)",
        "WFInputType": "URL",
        "WFAskActionDefaultAnswer": text_token_string(
            OCR,
            {
                "{0, 1}": {
                    **shortcut_input_att,
                    "Aggrandizements": [
                        {
                            "Type": "WFCoercionVariableAggrandizement",
                            "CoercionItemClass": "WFURLContentItem",
                        }
                    ],
                }
            },
        ),
    },
})

# 4) Choix de la plateforme
actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
    "WFWorkflowActionParameters": {
        "UUID": menu_uuid,
        "GroupingIdentifier": menu_group,
        "WFControlFlowMode": 0,
        "WFMenuPrompt": "Ouvrir sur",
        "WFMenuItems": ["Apple Music", "Deezer", "Tidal", "Spotify"],
    },
})

for title, target in [
    ("Apple Music", "apple_music"),
    ("Deezer", "deezer"),
    ("Tidal", "tidal"),
    ("Spotify", "spotify"),
]:
    actions.append({
        "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
        "WFWorkflowActionParameters": {
            "GroupingIdentifier": menu_group,
            "WFControlFlowMode": 1,
            "WFMenuItemTitle": title,
        },
    })
    actions.append({
        "WFWorkflowActionIdentifier": "is.workflow.actions.gettext",
        "WFWorkflowActionParameters": {
            "UUID": U(),
            "CustomOutputName": "target",
            "WFTextActionText": target,
        },
    })

actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
    "WFWorkflowActionParameters": {
        "GroupingIdentifier": menu_group,
        "WFControlFlowMode": 2,
    },
})

# 5) POST convert — url = réponse Ask (pas ExtensionInput brut)
actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.downloadurl",
    "WFWorkflowActionParameters": {
        "UUID": download_uuid,
        "CustomOutputName": "Réponse API",
        "ShowHeaders": False,
        "WFHTTPMethod": "POST",
        "WFHTTPBodyType": "JSON",
        "WFURL": text_token_string(
            OCR, {"{0, 1}": attachment_ref(api_text_uuid, "URL API de conversion")}
        ),
        "WFHTTPHeaders": {
            "Value": {
                "WFDictionaryFieldValueItems": [
                    {
                        "WFItemType": 0,
                        "WFKey": text_token_string("Content-Type"),
                        "WFValue": text_token_string("application/json"),
                    }
                ]
            },
            "WFSerializationType": "WFDictionaryFieldValue",
        },
        "WFJSONValues": {
            "Value": {
                "WFDictionaryFieldValueItems": [
                    {
                        "WFItemType": 0,
                        "WFKey": text_token_string("url"),
                        "WFValue": text_token_string(
                            OCR,
                            {
                                "{0, 1}": aggrandized_attachment(
                                    ask_uuid, "Lien du morceau", "WFStringContentItem"
                                )
                            },
                        ),
                    },
                    {
                        "WFItemType": 0,
                        "WFKey": text_token_string("target"),
                        "WFValue": text_token_string(
                            OCR,
                            {"{0, 1}": attachment_ref(menu_uuid, "Menu Result")},
                        ),
                    },
                ]
            },
            "WFSerializationType": "WFDictionaryFieldValue",
        },
    },
})

actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.detect.dictionary",
    "WFWorkflowActionParameters": {
        "UUID": dict_from_uuid,
        "CustomOutputName": "Dictionnaire de réponse",
        "WFInput": {
            "Value": attachment_ref(download_uuid, "Réponse API"),
            "WFSerializationType": "WFTextTokenAttachment",
        },
    },
})

actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.getvalueforkey",
    "WFWorkflowActionParameters": {
        "UUID": getvalue_uuid,
        "CustomOutputName": "URL résultat",
        "WFDictionaryKey": "url",
        "WFInput": {
            "Value": attachment_ref(dict_from_uuid, "Dictionnaire de réponse"),
            "WFSerializationType": "WFTextTokenAttachment",
        },
    },
})

actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.setclipboard",
    "WFWorkflowActionParameters": {
        "UUID": result_clipboard_uuid,
        "WFInput": {
            "Value": attachment_ref(getvalue_uuid, "URL résultat"),
            "WFSerializationType": "WFTextTokenAttachment",
        },
    },
})

actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
    "WFWorkflowActionParameters": {
        "UUID": open_menu_uuid,
        "GroupingIdentifier": open_menu_group,
        "WFControlFlowMode": 0,
        "WFMenuPrompt": "Lien copié. Ouvrir maintenant ?",
        "WFMenuItems": ["Ouvrir", "OK"],
    },
})
actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
    "WFWorkflowActionParameters": {
        "GroupingIdentifier": open_menu_group,
        "WFControlFlowMode": 1,
        "WFMenuItemTitle": "Ouvrir",
    },
})
actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.openurl",
    "WFWorkflowActionParameters": {
        "UUID": open_uuid,
        "WFInput": {
            "Value": attachment_ref(getvalue_uuid, "URL résultat"),
            "WFSerializationType": "WFTextTokenAttachment",
        },
    },
})
actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
    "WFWorkflowActionParameters": {
        "GroupingIdentifier": open_menu_group,
        "WFControlFlowMode": 1,
        "WFMenuItemTitle": "OK",
    },
})
actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
    "WFWorkflowActionParameters": {
        "GroupingIdentifier": open_menu_group,
        "WFControlFlowMode": 2,
    },
})

workflow = {
    "WFQuickActionSurfaces": [],
    "WFWorkflowActions": actions,
    "WFWorkflowClientRelease": "3.0",
    "WFWorkflowClientVersion": "2607.0.3",
    "WFWorkflowHasOutputFallback": False,
    "WFWorkflowHasShortcutInputVariables": True,
    "WFWorkflowIcon": {
        "WFWorkflowIconGlyphNumber": 59820,
        "WFWorkflowIconStartColor": 431817727,
    },
    "WFWorkflowImportQuestions": [],
    "WFWorkflowInputContentItemClasses": [
        "WFURLContentItem",
        "WFStringContentItem",
    ],
    "WFWorkflowMinimumClientVersion": 900,
    "WFWorkflowMinimumClientVersionString": "900",
    "WFWorkflowOutputContentItemClasses": [],
    "WFWorkflowTypes": ["ActionExtension", "NCWidget", "WatchKit"],
    # Sans entrée Share Sheet → lit le presse-papiers automatiquement
    "WFWorkflowNoInputBehavior": {
        "Name": "WFWorkflowNoInputBehaviorGetClipboard",
        "Parameters": {},
    },
}

out.write_bytes(plistlib.dumps(workflow, fmt=plistlib.FMT_BINARY))
print("unsigned →", out)
print("API Convert URL =", endpoint)
PY

SIGNED="$ROOT/static/shortcut/UNI-SHARE.shortcut"
shortcuts sign -m anyone -i "$ROOT/static/shortcut/UNI-SHARE-unsigned.shortcut" -o "$SIGNED"
echo "signed → $SIGNED"
echo "Ouvre le fichier et clique sur Ajouter le raccourci si demandé."
