# critic
This is a webapp to enable digital textual criticism.

# HOW TO BUILD
## tailwind 4.1.11
use in critic subdirectory (the main app code)
```
./tailwindcss -i style/input.css -o style/output.css --watch
```

## cargo leptos serve
Use in the main directory (the workspace dir)
```
cargo leptos serve
```
Use the `--release` toggle for release.

# Reverse Proxying critic
You need to set a relatively high `client_max_body_size` (for `nginx`).
This is currently `150m` for endpoints under `/upload`. Other paths do not need large `client_max_body_size`.

# Known Bugs
- When changing a manuscript name, a manual page reload is required to refetch the manuscript name from the server - the name in the left-hand MS-list will not be updated until then

# TODOs - next
## Admin page for adding manuscripts
### page edit
    - this can probably wait a bit
- change name
- upload new image
- link to fullsize image
- show preview image

## Overview of required transcriptions

## Saving transcriptions
- new page for transcribing a manuscript

## styling
### navbar
### get-started
### admin landing page

# TODOs - Editor
## automatically set lr/rl based on language

## Output styles
### Render to some html that is easily digestible
### allow copying the produced xml
    - both in a new component, that has three tabs - the editor is one of them

## Merge Blocks
- up and down
- das ist nicht immer erlaubt - ggf ist es besser, das einfach nicht zu erlauben und automatisch zu machen??

## Styling

# TODOs - General
## setup docker for code deployment later

## create admin pages for
### adding manuscripts / folios
### editing versification schemes

## Create user-facing pages for
### transcribing
- plus a large editor
- publish transcription
### reconciliation
- this will require the reconciliation editor
- this in turn will require the reconciliation differ
### overview of required transcriptions
- search bar, links to transcribe/source
### overview of required reconciliations
- search bar, links to reconcile/source
### landing page

# TODOs - actual website
High level landing page for tanakhcc.org

# TODOs - diff
We will need a good multi-diff algorithm
take the idea from multidiff:
- use wu-diff-by-hash on two inputs
- create a mapping "content"-"present-in-inputs" like in multidiff

This multi-diff will be used in collation, but also during reconciliation

# TODOs - Branches
## master
- the default branch that end users see and pull the data from
- used for releases
    - releases are tagged commits
- high branch protection, forced CI, ...

## `rec/<source>/<user>`
- active reconciliations that `<user>` wants to be final for `<source>`
- highly incentivise users to not open new reconciliations when ones already exist for a source
- merged into `master` when accepted

# TODOs - Workflow
- The user creates a transcription (using the editor)
    - one version (current) for each source and user is kept
- The user can "publish" a transcription
    - this adds an older version for that user and source
    - all older versions are visible to everyone
- Another or the same user can then create Reconciliations
    - Select a source (a single page/image)
    - all newest transcriptions for that source will be displayed
    - create one reconciled version (we need a new editor for this)
        - always pick the majority view for blocks by default
        - allow picking blocks from different versions (copy them over from one displayer into the final editor)
        - allow a full editor for the final version
    - create a new branch on top of master ("rec/source/user"), setting the transcript file for that source
    - create a MR from this branch onto some working branch ("critic/accepted"), with a message containing:
        - (ignore blocks that are equal)
        - how often was majority decision taken
        - how often was minority decision taken
        - how often was a completely different decision taken
- Releases then take multiple branches (e.g. for one manuscript, or for one logical part of text)
    - all these branches are merged into one big feature branch
    - that branch is rebased onto master

## Q&A
Using normal gitlab issues

## Release Versions
Using normal gitlab releases

## Should we host a matrix server as well for better chats?
Would be nice, but require a central LDAP for auth management

## Manuscripts, Pages
- Manuscripts = a group of folios with meta-information (representable in TEI format)
- Pages = File on the OS level (a single xml file)

# Source of truth
Source of truth is mixed.
DB is used authoritatively for:
- session stores (local sqlite)
- current state of published files
gitlab is used authoritatively for everything else:
- actual transcription data (after reconciliation)
- (source metadata - this is part of the XML files for transcriptions)

## auto-rebuild
- DB is rebuilt every now and then (daily??) from gitlab
    - we check consistency of gitlab by building into a dev-db
    - if that works, we build into the actual db

## consistency check
- check that all xml files are parsable in our subscheme
- check that all metadata for a source is consistent

