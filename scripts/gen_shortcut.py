#!/usr/bin/env python3
"""Generate an unsigned UNI-SHARE Shortcut plist (binary)."""
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


def main() -> None:
    endpoint = sys.argv[1]
    out = Path(sys.argv[2])

    api_u = U()
    clip_u = U()
    ask_u = U()
    menu_u = U()
    menu_g = U()
    req_u = U()
    dl_u = U()
    dict_u = U()
    url_u = U()
    err_u = U()
    if_g = U()
    open_u = U()
    clip_out_u = U()

    shortcut_in = {"Type": "ExtensionInput", "VariableName": "Shortcut Input"}
    actions: list[dict] = []

    # 1) API endpoint
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.gettext",
            "WFWorkflowActionParameters": {
                "UUID": api_u,
                "CustomOutputName": "URL API de conversion",
                "WFTextActionText": endpoint,
            },
        }
    )

    # 2) Clipboard (also NoInputBehaviorGetClipboard)
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.getclipboard",
            "WFWorkflowActionParameters": {
                "UUID": clip_u,
                "CustomOutputName": "Presse-papiers",
            },
        }
    )

    # 3) Ask / confirm URL — default Shortcut Input (share or clipboard-via-no-input)
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.ask",
            "WFWorkflowActionParameters": {
                "UUID": ask_u,
                "CustomOutputName": "Lien du morceau",
                "WFAskActionPrompt": "Lien du morceau (partage, presse-papiers ou colle)",
                "WFInputType": "URL",
                "WFAskActionDefaultAnswer": text_token(
                    OCR, {"{0, 1}": shortcut_in}
                ),
            },
        }
    )

    # 4) Platform menu
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
        actions.append(
            {
                "WFWorkflowActionIdentifier": "is.workflow.actions.gettext",
                "WFWorkflowActionParameters": {
                    "UUID": U(),
                    "CustomOutputName": "cible",
                    "WFTextActionText": target,
                },
            }
        )
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.choosefrommenu",
            "WFWorkflowActionParameters": {
                "GroupingIdentifier": menu_g,
                "WFControlFlowMode": 2,
            },
        }
    )

    # 5) Request dictionary
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.dictionary",
            "WFWorkflowActionParameters": {
                "UUID": req_u,
                "CustomOutputName": "Requête",
                "WFItems": {
                    "Value": {
                        "WFDictionaryFieldValueItems": [
                            {
                                "WFItemType": 0,
                                "WFKey": text_token("url"),
                                "WFValue": text_token(
                                    OCR,
                                    {
                                        "{0, 1}": out_ref(
                                            ask_u,
                                            "Lien du morceau",
                                            "WFStringContentItem",
                                        )
                                    },
                                ),
                            },
                            {
                                "WFItemType": 0,
                                "WFKey": text_token("target"),
                                "WFValue": text_token(
                                    OCR,
                                    {
                                        "{0, 1}": out_ref(
                                            menu_u,
                                            "Menu Result",
                                            "WFStringContentItem",
                                        )
                                    },
                                ),
                            },
                        ]
                    },
                    "WFSerializationType": "WFDictionaryFieldValue",
                },
            },
        }
    )

    # 6) POST
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.downloadurl",
            "WFWorkflowActionParameters": {
                "UUID": dl_u,
                "CustomOutputName": "Réponse API",
                "ShowHeaders": False,
                "WFHTTPMethod": "POST",
                "WFHTTPBodyType": "JSON",
                "WFURL": text_token(
                    OCR, {"{0, 1}": out_ref(api_u, "URL API de conversion")}
                ),
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
                    "Value": out_ref(req_u, "Requête"),
                    "WFSerializationType": "WFTextTokenAttachment",
                },
            },
        }
    )

    # 7) Parse
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

    # 8) If result contains http → clipboard + notify + open; else show error
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
                "UUID": clip_out_u,
                "WFInput": {
                    "Value": out_ref(url_u, "URL résultat"),
                    "WFSerializationType": "WFTextTokenAttachment",
                },
            },
        }
    )
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.notification",
            "WFWorkflowActionParameters": {
                "UUID": U(),
                "WFNotificationActionTitle": "UNI-SHARE",
                "WFNotificationActionBody": "Lien copié dans le presse-papiers",
            },
        }
    )
    actions.append(
        {
            "WFWorkflowActionIdentifier": "is.workflow.actions.openurl",
            "WFWorkflowActionParameters": {
                "UUID": open_u,
                "WFInput": {
                    "Value": out_ref(url_u, "URL résultat", "WFURLContentItem"),
                    "WFSerializationType": "WFTextTokenAttachment",
                },
            },
        }
    )

    # Otherwise
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
