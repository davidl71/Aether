#!/usr/bin/env python3
"""
Script to create a NotebookLM notebook with all resources.
This script generates a JSON file with all resources ready to add to NotebookLM.
"""

import json
import os

# All resources to add to NotebookLM
RESOURCES = {
    "notebook_name": "TWS Automated Trading - Complete Resources",
    "description": "Complete knowledge base for TWS Automated Trading project including GitHub repository documentation, TWS API implementation videos, trading tutorials, and implementation articles.",
    "topics": [
        "TWS API",
        "Interactive Brokers",
        "Options Trading",
        "Box Spreads",
        "C++ Implementation",
        "Socket Programming",
        "Trading Strategies",
        "API Integration"
    ],
    "tags": [
        "tws-api",
        "trading",
        "options",
        "documentation",
        "youtube",
        "tutorial",
        "ib-api",
        "c++",
        "socket-implementation",
        "boost-asio"
    ],
    "use_cases": [
        "Research TWS API implementation",
        "Summarize YouTube tutorials",
        "Create documentation from videos",
        "Research trading strategies",
        "Get code examples for TWS API"
    ],
    "content_types": [
        "documentation",
        "tutorial",
        "video",
        "article",
        "code examples",
        "implementation guide"
    ],
    "resources": [
        {
            "type": "github",
            "url": "https://github.com/davidl71/ib_box_spread_full_universal",
            "description": "GitHub repository with 47+ documentation files"
        },
        {
            "type": "youtube",
            "url": "https://www.youtube.com/watch?v=n-9bdREECTQ",
            "description": "Essential Components of TWS API Programs"
        },
        {
            "type": "youtube",
            "url": "https://www.youtube.com/watch?v=5moyX0qwkCA",
            "description": "Trading & Options Video 1"
        },
        {
            "type": "youtube",
            "url": "https://www.youtube.com/watch?v=hJ7ewxQVhJw&list=PLePBf4ZtCKhpovbIfv85Ks3-rvyNtAwsg&index=2",
            "description": "Trading & Options Video 2 (Playlist 1)"
        },
        {
            "type": "youtube",
            "url": "https://www.youtube.com/watch?v=4zpYhHn5p90&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2",
            "description": "Trading & Options Video 3 (Playlist 2)"
        },
        {
            "type": "youtube",
            "url": "https://www.youtube.com/watch?v=rC02897uiuc&list=PLePBf4ZtCKhpovbIfv85Ks3-rvyNtAwsg&index=2",
            "description": "Trading & Options Video 4 (Playlist 1)"
        },
        {
            "type": "youtube",
            "url": "https://www.youtube.com/watch?v=ZxwdTgMY44g&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2",
            "description": "Trading & Options Video 5 (Playlist 2)"
        },
        {
            "type": "youtube",
            "url": "https://www.youtube.com/watch?v=ICZH89GdUGQ&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2",
            "description": "Trading & Options Video 6 (Playlist 2)"
        },
        {
            "type": "youtube",
            "url": "https://www.youtube.com/watch?v=W6OJy32sE_g&list=PLePBf4ZtCKhqBGXyB1v4xvZiMju6NQlIw&index=2",
            "description": "Trading & Options Video 7 (Playlist 2)"
        },
        {
            "type": "article",
            "url": "https://www.vitaltrades.com/2024/02/02/making-a-c-interactive-brokers-tws-client-with-a-custom-socket-implementation/",
            "description": "Making a C++ Interactive Brokers TWS Client with a Custom Socket Implementation"
        }
    ]
}

def generate_resources_file():
    """Generate a JSON file with all resources."""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)
    output_file = os.path.join(project_root, "docs", "notebooklm_resources.json")

    with open(output_file, 'w') as f:
        json.dump(RESOURCES, f, indent=2)

    print(f"✅ Generated resources file: {output_file}")
    return output_file

def print_urls():
    """Print all URLs for easy copy-paste."""
    print("\n" + "="*80)
    print("All URLs to Add to NotebookLM (Copy-Paste Ready)")
    print("="*80 + "\n")

    for i, resource in enumerate(RESOURCES["resources"], 1):
        print(f"{i}. {resource['url']}")
        print(f"   Type: {resource['type']}")
        print(f"   Description: {resource['description']}")
        print()

def print_summary():
    """Print summary of resources."""
    print("\n" + "="*80)
    print("NotebookLM Resources Summary")
    print("="*80 + "\n")
    print(f"Notebook Name: {RESOURCES['notebook_name']}")
    print(f"Description: {RESOURCES['description']}")
    print(f"\nTotal Resources: {len(RESOURCES['resources'])}")
    print(f"  - GitHub Repository: 1")
    print(f"  - YouTube Videos: {sum(1 for r in RESOURCES['resources'] if r['type'] == 'youtube')}")
    print(f"  - Articles: {sum(1 for r in RESOURCES['resources'] if r['type'] == 'article')}")
    print(f"\nTopics: {', '.join(RESOURCES['topics'])}")
    print(f"Tags: {', '.join(RESOURCES['tags'])}")
    print()

if __name__ == "__main__":
    print("="*80)
    print("NotebookLM Resources Generator")
    print("="*80)

    # Generate resources file
    output_file = generate_resources_file()

    # Print summary
    print_summary()

    # Print URLs
    print_urls()

    print("="*80)
    print("Next Steps:")
    print("="*80)
    print("1. Go to https://notebooklm.google.com")
    print("2. Click '+ New' to create a new notebook")
    print(f"3. Name it: {RESOURCES['notebook_name']}")
    print("4. Add each URL from the list above")
    print("5. Wait for all resources to process (may take 10-20 minutes)")
    print("6. Share the notebook and copy the link")
    print("7. Return to Cursor and say: 'Add [link] to library tagged [tags]'")
    print("="*80)
