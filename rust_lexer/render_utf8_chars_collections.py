#!/usr/bin/python3
import re
import requests

XID_CONTINUE_URL = r"https://util.unicode.org/UnicodeJsps/list-unicodeset.jsp?a=%5B%3AXID_Continue%3A%5D&g=&i="
XID_START_URL = r"https://util.unicode.org/UnicodeJsps/list-unicodeset.jsp?a=%5B%3AXID_Start%3A%5D&g=&i="

r = requests.get(XID_START_URL)
codes = re.findall(r"U\+([0-9a-fA-F]+)", r.text)
print(" | ".join(f"'\\u{{{c.upper()}}}'" for c in codes))