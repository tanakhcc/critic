# critic
This is a webapp to enable digital textual criticism.

# TODOs - Editor
## How do we display global errors?

## Other Blocks (copy the list from ATG)
for each:
- implement behaviour
- implement view
- implement undo/redo if required
### Anchors
### Corrections

## Make Break a dropdown instead of a raw string input

## Merge Blocks
- up and down

## Styling

## Ribbon with the commands for which keycommands exist

## bugs
- behaviour for lacuna and break sometimes does not work correctly (getting wrong index and splitting the field incorrectly)

# TODOs - General
## get the currently logged in user into leptos (frontend) somehow?
we want to be able to show who the user is and such
- maybe just have resources that pull that from the server??
## setup docker for code deployment later

## emit xml files for transcripts

# TODOs - Workflow
- The user creates a transcription (using the editor)
    - one version (current) for each source and user is kept
- The user can "publish" a transcription
    - this adds an older version for that user and source
    - all older versions are visible to everyone
- Another or the same user can then create Reconciliations
    - Select a source
    - Select any number of base transcriptions from the published transcriptions for that source
    - create one reconciled version (we need a new editor for this)
    - create a new branch on top of master ("rec/user/source"), setting the transcript file for that source and a merge request into master for it
    - (reconciliation is equivalent to MR - when the MR is merged, the reconciliation no longer exists)
    - debate on reconciliations happens in the MR in gitlab like for a normal MR

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

