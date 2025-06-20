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
## setup oauth2 flow with our own gitlab
### setup host entries on my machine to make this work with proper https
### setup server that intercepts the call to /oauth/redirect
- we should get an authorization-code from that
### trade for token
### now have an ephemeral table (SQLite??) that holds user session tokens (aka oauth tokens and refresh tokens) for users
### write some middleware to do this
#### redirect to oauth if no session exists
#### do the refresh-dance if it exists but is expired
#### give access to the page if a session exists and is active

## setup docker for code deployment later

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

