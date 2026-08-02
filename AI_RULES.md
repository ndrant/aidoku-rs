# AI_RULES.md

# =============================================================================
# PROJECT IDENTITY
# =============================================================================

This repository is NOT an application.

This repository is NOT an iOS project.

This repository is NOT an Android project.

This repository exists ONLY for developing Aidoku Source Extensions.

The Aidoku application already exists.

This repository only produces .aix extension packages.

Every implementation must support this objective.

Never generate application code.

Never generate UI code.

Never generate SwiftUI.

Never generate UIKit.

Never generate iOS application features.

Only generate extension-related code.

===============================================================================
PROJECT GOAL
===============================================================================

The only purpose of this repository is to build production-grade Aidoku Sources.

The final output of this repository is:

.aix

Extension Package

Nothing else.

===============================================================================
WHAT THIS PROJECT IS
===============================================================================

This project contains:

✓ Rust source code

✓ aidoku-rs implementation

✓ HTML parser

✓ JSON parser

✓ Website parser

✓ Network layer

✓ Resource files

✓ Filters

✓ Settings

✓ Packaging

===============================================================================
WHAT THIS PROJECT IS NOT
===============================================================================

Never generate:

iOS App

SwiftUI

UIKit

Navigation

Reader UI

Bookshelf

Downloads

History

Bookmarks

Favorites

Database

Authentication Screen

Splash Screen

Settings Screen

Tab Bar

Any application feature

Those belong to Aidoku itself.

===============================================================================
PROJECT OUTPUT
===============================================================================

The only output of this repository is:

Source Code

↓

cargo build

↓

WebAssembly

↓

aidoku package

↓

package.aix

The repository must never attempt to replace Aidoku.

===============================================================================
SUPPORTED RESPONSIBILITIES
===============================================================================

The extension is responsible only for:

Searching manga

Fetching manga details

Fetching chapter list

Fetching page list

Fetching homepage

Fetching latest updates

Fetching popular manga

Fetching genres

Fetching filters

Providing settings

Handling website requests

Parsing HTML

Parsing JSON

Returning Aidoku models

Nothing else.

===============================================================================
FORBIDDEN RESPONSIBILITIES
===============================================================================

Never implement:

Reader

Reader Controls

Image Viewer

Download Manager

Offline Reading

Library

Bookmarks

Reading History

Account System

Sync

Cloud Backup

Database

Notification

Theme

Dark Mode

Login UI

Navigation

Cache Database

Anything outside the extension scope.

===============================================================================
ARCHITECTURE
===============================================================================

Website

↓

Network

↓

Parser

↓

Models

↓

Aidoku Source

↓

Aidoku App

Aidoku App is outside this repository.

Do not implement anything beyond the Source layer.

===============================================================================
WEBSITE SUPPORT
===============================================================================

Every website must remain isolated.

src/sites/

    komikcast/

    natsu/

Each site owns:

search.rs

detail.rs

chapter.rs

pages.rs

parser.rs

selectors.rs

No site depends on another site.

===============================================================================
RESOURCE FILES
===============================================================================

Every source must include:

source.json

filters.json

settings.json

icon.png

The AI should maintain these resources whenever necessary.

===============================================================================
BUILD TARGET
===============================================================================

Supported build commands:

cargo build

cargo fmt

cargo clippy

cargo test

aidoku package

The expected final artifact is:

package.aix

===============================================================================
QUALITY REQUIREMENTS
===============================================================================

Every implementation must:

Compile successfully

Avoid placeholder code

Avoid TODO

Avoid FIXME

Avoid mock logic

Avoid fake implementations

Be production ready

Be modular

Be documented

Be testable

===============================================================================
PARSER RULES
===============================================================================

Never parse HTML with regex.

Always use CSS selectors.

Never duplicate selectors.

Never perform network requests inside parser modules.

Never return invalid Aidoku models.

===============================================================================
NETWORK RULES
===============================================================================

Every HTTP request must use the shared network client.

Support:

Timeout

Retry

Cookies

Headers

Compression

User-Agent

Redirect

Never duplicate networking logic.

===============================================================================
ERROR HANDLING
===============================================================================

Every public function returns Result<T, AidokuError>.

Every failure includes context.

No unwrap().

No panic!().

No silent failures.

===============================================================================
FINAL OBJECTIVE
===============================================================================

The repository should become a professional collection of Aidoku Source extensions.

Every generated code should move the project closer to a stable, production-quality .aix package.

Never generate application code.

Only generate extension code.

===============================================================================
END
===============================================================================