#!/usr/bin/env python3
"""Generate unsigned UNI-SHARE Shortcut — per-menu-branch convert (reliable params)."""
from __future__ import annotations

import plistlib
import sys
import uuid
from pathlib import Path

OCR = "\ufffc"


def U() -> str:
    return str(uuid.uuid4()).upper()


def text_token(text: str, attachments: dict | None = None) -> dict:
    return {
        "Value": {"string": text, "attachmentsByRange": attachments or {}},
        "WFSerializationType": "WFTextTokenString",
    }


def dict_token_value(attachments: dict) -> dict:
    """Dictionary field value from a magic variable."""
    return text_token(OCR, attachments)


def out_ref(uid: str, name: str, coerce: str | None = None) -> dict:
    ref: dict = {"OutputUUID": uid, "OutputName": name, "Type": "ActionOutput"}
    if coerce:
        ref["Aggrandizements"] = [
            {
                "Type": "WFCoercionVariableAggrandizement",
                "CoercionItemClass": coerce,
            }
        ]
    return ref


def ask_ref(ask_u: str) -> dict:
    return out_ref(ask_u, "Lien du morceau", "WFStringContentItem")


def convert_actions(endpoint: str, ask_u: str) -> list[dict]:
    """POST convert → parse → clipboard + open, or show error."""
    dl_u = U()
    dict_u = U()
    url_u = U()
    err_u = U()
    if_g = U()
    actions: list[dict] = []

    # target is filled by caller via closure — we build download with placeholder
    # replaced per branch; this helper receives target string.
    return actions  # unused


def branch_convert(endpoint: str, ask_u: str, target: str) -> list[dict]:
    dl_u = U()
    dict_u = U()
    url_u = U()
    err_u = U()
    if_g = U()
    actions: list[dict] = []

    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.downloadurl",
            "WFWorkflowActionParameters": {
                "UUID": dl_u,
                "CustomOutputName": "Réponse API",
                "ShowHeaders": False,
                "WFHTTPMethod": "POST",
                "WFHTTPBodyType": "JSON",
                # Plain string URL — no empty magic var
                "WFURL": endpoint,
                "WFHTTPHeaders": {
                    "Value": {
                        "WFDictionaryFieldValueItems": [
                            {
                                "WFItemType": 0,
                                "WFKey": text_token("Content-Type"),
                                "WFValue": text_token("application/json"),
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
                                "WFKey": text_token("url"),
                                "WFValue": dict_token_value(
                                    {"{0, 1}": ask_ref(ask_u)}
                                ),
                            },
                            {
                                "WFItemType": 0,
                                "WFKey": text_token("target"),
                                "WFValue": text_token(target),
                            },
                        ]
                    },
                    "WFSerializationType": "WFDictionaryFieldValue",
                },
            },
        }
    )

    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.detect.dictionary",
            "WFWorkflowActionParameters": {
                "UUID": dict_u,
                "CustomOutputName": "Dictionnaire",
                "WFInput": {
                    "Value": out_ref(dl_u, "Réponse API"),
                    "WFSerializationType": "WFTextTokenAttachment",
                },
            },
        }
    )
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.getvalueforkey",
            "WFWorkflowActionParameters": {
                "UUID": url_u,
                "CustomOutputName": "URL résultat",
                "WFDictionaryKey": "url",
                "WFInput": {
                    "Value": out_ref(dict_u, "Dictionnaire"),
                    "WFSerializationType": "WFTextTokenAttachment",
                },
            },
        }
    )

    # If contains http
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.conditional",
            "WFWorkflowActionParameters": {
                "UUID": U(),
                "GroupingIdentifier": if_g,
                "WFControlFlowMode": 0,
                "WFCondition": 4,
                "WFConditionalActionString": "http",
                "WFInput": {
                    "Value": out_ref(url_u, "URL résultat", "WFStringContentItem"),
                    "WFSerializationType": "WFTextTokenAttachment",
                },
            },
        }
    )
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.setclipboard",
            "WFWorkflowActionParameters": {
                "UUID": U(),
                "WFInput": {
                    "Value": out_ref(url_u, "URL résultat"),
                    "WFSerializationType": "WFTextTokenAttachment",
                },
            },
        }
    )
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.openurl",
            "WFWorkflowActionParameters": {
                "UUID": U(),
                "WFInput": {
                    "Value": out_ref(url_u, "URL résultat", "WFURLContentItem"),
                    "WFSerializationType": "WFTextTokenAttachment",
                },
            },
        }
    )

    # Otherwise show API message
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.conditional",
            "WFWorkflowActionParameters": {
                "UUID": U(),
                "GroupingIdentifier": if_g,
                "WFControlFlowMode": 1,
            },
        }
    )
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.getvalueforkey",
            "WFWorkflowActionParameters": {
                "UUID": err_u,
                "CustomOutputName": "Message erreur",
                "WFDictionaryKey": "message",
                "WFInput": {
                    "Value": out_ref(dict_u, "Dictionnaire"),
                    "WFSerializationType": "WFTextTokenAttachment",
                },
            },
        }
    )
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.showresult",
            "WFWorkflowActionParameters": {
                "UUID": U(),
                "WFInput": {
                    "Value": out_ref(err_u, "Message erreur", "WFStringContentItem"),
                    "WFSerializationType": "WFTextTokenAttachment",
                },
            },
        }
    )
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.conditional",
            "WFWorkflowActionParameters": {
                "UUID": U(),
                "GroupingIdentifier": if_g,
                "WFControlFlowMode": 2,
            },
        }
    )
    return actions


def main() -> None:
    endpoint = sys.argv[1].rstrip("/")
    if not endpoint.endswith("/api/v1/convert"):
        endpoint = endpoint + "/api/v1/convert"
    out = Path(sys.argv[2])

    ask_u = U()
    menu_g = U()
    menu_u = U()
    clip_u = U()

    actions: list[dict] = []

    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.getclipboard",
            "WFWorkflowActionParameters": {
                "UUID": clip_u,
                "CustomOutputName": "Presse-papiers",
            },
        }
    )

    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.ask",
            "WFWorkflowActionParameters": {
                "UUID": ask_u,
                "CustomOutputName": "Lien du morceau",
                "WFAskActionPrompt": "Lien du morceau (partage, presse-papiers ou colle)",
                "WFInputType": "URL",
                "WFAskActionDefaultAnswer": text_token(
                    OCR,
                    {"{0, 1}": {"Type": "ExtensionInput", "VariableName": "Shortcut Input"}},
                ),
            },
        }
    )

    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
            "WFWorkflowActionParameters": {
                "UUID": menu_u,
                "GroupingIdentifier": menu_g,
                "WFControlFlowMode": 0,
                "WFMenuPrompt": "Ouvrir sur",
                "WFMenuItems": ["Apple Music", "Deezer", "Tidal", "Spotify"],
            },
        }
    )

    for title, target in [
        ("Apple Music", "apple_music"),
        ("Deezer", "deezer"),
        ("Tidal", "tidal"),
        ("Spotify", "spotify"),
    ]:
        actions.append(
            {
                "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
                "WFWorkflowActionParameters": {
                    "GroupingIdentifier": menu_g,
                    "WFControlFlowMode": 1,
                    "WFMenuItemTitle": title,
                },
            }
        )
        actions.extend(branch_convert(endpoint, ask_u, target))

    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
            "WFWorkflowActionParameters": {
                "GroupingIdentifier": menu_g,
                "WFControlFlowMode": 2,
            },
        }
    )

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
        "WFWorkflowNoInputBehavior": {
            "Name": "WFWorkflowNoInputBehaviorGetClipboard",
            "Parameters": {},
        },
    }

    out.write_bytes(plistlib.dumps(workflow, fmt=plistlib.FMT_BINARY))
    print(f"wrote {out} ({out.stat().st_size} bytes, {len(actions)} actions)")


if __name__ == "__main__":
    main()
