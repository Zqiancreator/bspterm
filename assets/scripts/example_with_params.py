#!/usr/bin/env python3
"""
Example script with parameter support.

This script demonstrates how to use parameters in BSPTerm scripts.
Parameters are declared using the @params block and accessed via the params object.

@params
- message: string
  description: Message to display
  required: true
  default: "Hello"

- count: number
  description: Repeat count
  default: 3

- uppercase: boolean
  description: Convert to uppercase
  default: false

- style: choice
  description: Output style
  choices: ["plain", "boxed", "numbered"]
  default: "plain"
@end_params
"""

from bspterm import params

# Get parameters with defaults
message = params.message
count = params.get("count", 3)
uppercase = params.get("uppercase", False)
style = params.get("style", "plain")

# Process message
text = message.upper() if uppercase else message

# Output based on style
print(f"Style: {style}, Count: {count}, Uppercase: {uppercase}")
print("-" * 40)

for i in range(count):
    if style == "boxed":
        print(f"| {text} |")
    elif style == "numbered":
        print(f"{i + 1}. {text}")
    else:
        print(text)

print("-" * 40)
print("Done!")
