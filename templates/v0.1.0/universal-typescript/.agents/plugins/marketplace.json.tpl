{
  "name": {{MARKETPLACE_JSON}},
  "interface": {
    "displayName": {{DISPLAY_NAME_JSON}}
  },
  "plugins": [
    {
      "name": {{NAME_JSON}},
      "source": {
        "source": "local",
        "path": "./"
      },
      "policy": {
        "installation": "AVAILABLE",
        "authentication": "ON_INSTALL"
      },
      "category": "Developer Tools"
    }
  ]
}
