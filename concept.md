# Concept



## Service

The service is the one communicating with the google drive API. It will check for updates when it is asked and it will download files when requested. 
It also uploads changes to local files.
It also keeps track of what files should be kept locally and what files only get cached locally when accessed (important for offline access).

It needs to be able to do the following:
- Check for updates
- Download files
- Resolve a file path to a file id
- Upload changes
- Get children of a folder



When the service is asked for metadata or content of a file it gets it from the API if it does not have a local copy already and the local copy is not too old (configurable).
It then creates a file in its own folder with the filename being the file id with no extension. If only the metadata is requested it will create a file with the extension .meta.

When the user wants to read the file content, it should (first ask the service to check for changes? and then) see if the file in the service folder exists with the id. If it doesn't it should
request the content from the service and wait for its completion.


