# Concept



## Service

The service is the one communicating with the Google Drive API. It will check for updates when it is asked, and it will 
download files when requested.

It also uploads changes to local files.

It also keeps track of what files should be kept locally and what files only get cached locally when accessed (important 
for offline access).

It needs to be able to do the following:
- Check for updates
- Download files
- Resolve a file path to a file id
- Upload changes
- Get children of a folder

---

When the service is asked for metadata or content of a file it gets it from the API if it does not have a local copy 
already and the local copy is not too old (configurable). It then creates a file in its cache or download folder with 
the filename being the file id with no extension. The metadata will be on a file with the extension '.meta' in 
the meta folder, this file will always be created (or updated), no matter if only the content or only the metadata was 
requested. 

The meta file will be a simple UTF-8 file, containing a number from 0 to 2:

- 0: only metadata was fetched, no content available
- 1: content was fetched to cache
- 2: content was fetched to downloads

---

When the user wants to read the file content, it should request the file content from the service, which should trigger 
an update check on the service side, and then read the meta file to determine if it is in cache or downloads. If it is 
neither something really went wrong and the program might not be online or the requested file might not exist anymore.
