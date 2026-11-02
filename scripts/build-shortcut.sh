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
menu_uuid = U()
menu_group = U()
download_uuid = U()
getvalue_uuid = U()
open_uuid = U()
OCR = "\ufffc"

def text_token_string(text, attachments=None):
    return {
        "Value": {"string": text, "attachmentsByRange": attachments or {}},
        "WFSerializationType": "WFTextTokenString",
    }

def attachment_ref(output_uuid, output_name, typ="ActionOutput"):
    return {"OutputUUID": output_uuid, "OutputName": output_name, "Type": typ}

shortcut_input_att = {"Type": "ExtensionInput", "VariableName": "Shortcut Input"}
actions = []

actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.gettext",
    "WFWorkflowActionParameters": {
        "UUID": api_text_uuid,
        "CustomOutputName": "API Convert URL",
        "WFTextActionText": endpoint,
    },
})

actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
    "WFWorkflowActionParameters": {
        "UUID": menu_uuid,
        "GroupingIdentifier": menu_group,
        "WFControlFlowMode": 0,
        "WFMenuPrompt": "Open on",
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

actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.downloadurl",
    "WFWorkflowActionParameters": {
        "UUID": download_uuid,
        "CustomOutputName": "API Response",
        "ShowHeaders": False,
        "WFHTTPMethod": "POST",
        "WFHTTPBodyType": "JSON",
        "WFURL": text_token_string(
            OCR, {"{0, 1}": attachment_ref(api_text_uuid, "API Convert URL")}
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
                            OCR, {"{0, 1}": shortcut_input_att}
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

dict_from_uuid = U()
actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.detect.dictionary",
    "WFWorkflowActionParameters": {
        "UUID": dict_from_uuid,
        "CustomOutputName": "Response Dictionary",
        "WFInput": {
            "Value": attachment_ref(download_uuid, "API Response"),
            "WFSerializationType": "WFTextTokenAttachment",
        },
    },
})

actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.getvalueforkey",
    "WFWorkflowActionParameters": {
        "UUID": getvalue_uuid,
        "CustomOutputName": "Result URL",
        "WFDictionaryKey": "url",
        "WFInput": {
            "Value": attachment_ref(dict_from_uuid, "Response Dictionary"),
            "WFSerializationType": "WFTextTokenAttachment",
        },
    },
})

actions.append({
    "WFWorkflowActionIdentifier": "is.workflow.actions.openurl",
    "WFWorkflowActionParameters": {
        "UUID": open_uuid,
        "WFInput": {
            "Value": attachment_ref(getvalue_uuid, "Result URL"),
            "WFSerializationType": "WFTextTokenAttachment",
        },
    },
})

workflow = {
    "WFQuickActionSurfaces": [],
    "WFWorkflowActions": actions,
    "WFWorkflowClientRelease": "3.0",
    "WFWorkflowClientVersion": "2607.0.3",
    "WFWorkflowHasOutputFallback": False,
    "WFWorkflowHasShortcutInputVariables": False,
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
    "WFWorkflowNoInputBehavior": {
        "Name": "WFWorkflowNoInputBehaviorAskForInput",
        "Parameters": {
            "ItemClass": "WFURLContentItem",
            "SerializedParameters": {},
        },
    },
}

out.write_bytes(plistlib.dumps(workflow, fmt=plistlib.FMT_BINARY))
print("unsigned →", out)
print("API Convert URL =", endpoint)
PY

SIGNED="$ROOT/static/shortcut/UNI-SHARE.shortcut"
shortcuts sign -m anyone -i "$ROOT/static/shortcut/UNI-SHARE-unsigned.shortcut" -o "$SIGNED"
echo "signed → $SIGNED"
echo "Open it once and click Add Shortcut if Shortcuts asks."
