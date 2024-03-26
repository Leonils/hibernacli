# Cli backup proposal

## Context

Personal backups are complex and time-consuming. The goal of this project is to create a simple and easy-to-use tool to backup files and directories. This tool does not aim to just perform backups, but also to guide the user in the process of determining what to backup and how.

## Business concepts

### General concepts

As a user, I have several **devices** (my linux laptop, my server, my S3 bucket, my USB keys, my hard drives, my github account...). I may have valuable data on each of these devices. The part of the data of each device that is the reference one in the **primary data**. For instance, my SSH keys are primarily on my laptop, but my photos are primarily on my hard drive, and my app datas are on my server.

I want to backup my primary data on other devices. The data that is backed up is called **secondary data**. A storage device is typically made of **free space**, **primary data**, **secondary data** and **ignored data**.

All data is not created equal:

- **Privacy**: data might be highly sensitive (SSH keys), confidential (projects), personal (photos), or public (useful downloads, ...).
- **Criticality**: data might be critical (administrative files), important (projects), useful (old photos), or disposable (temporary files).
- **History**: data might need to be saved to keep an history (last X versions)

From these concepts, there are different class of backups:

- **Acceptable storages**: storages that are acceptable for the user to store secondary data. For instance, I may not want to store my SSH keys on a S3 bucket, but on a USB key.
- **Number of required copies**: some data might need to be stored in multiple copies (critical data), while some other data might be stored in a single copy (disposable data).
- **Encryption**: some data might need to be encrypted (private data), while some other data might not (public data).
- **History**: we might need to keep several versions of the data, or only the last one.

### Projects / Folders

When managing data, the **file** is not the right unit. I don't want to classify each file individually, but I want to classify "**projects**", or "**folders**". For instance, all photos of my last trip are part of the same project. They are homogeneous and constitute a whole.

Examples:

- Photos of my last trip
- Project XYZ
- All my health administrative files
- Each SSH key

For such a project, we want each time to determine **new files**, **modified files**, **deleted files**. The tool must help the user to determine the groups that have value to him, and to determine how to backup them.

Warning, there might be some exceptions. Let's consider a simple project structure:

```
project/
  .git/
  node_modules/
  src/
  package.json
  .env
  .gitignore
```

Here, the project might be important, gited, confidential, and need 3 copies. On the other hand, node_modules must be excluded from the backup, and .env is not as important as the project (because we just have to recreate it, and regenerate the tokens of the dev environment) but is still useful. However it is more confidential than the project itself.

### State of backups

A project might have several states of backups:

- **Ignored**: the project is not backed up and is not expected to be
- **Not categorized**: we don't know yet if the project is important to the user or not. It is not backed up yet. We need give full visibility to the user on the project to help him categorize it.
- **Backed up**: the project is tracked, and has some copies. It might not yet reach the target backup level, but for each copy:
  - **Up to date**: the project is on the copy and is up to date
  - **Outdated**

In the cli, the user must be able to overview his projects, their state, and his progress in categorizing and backing them up.

### Indexes

Comparing files is time-consuming (and for some storages costly). We need to keep indexes of the files in each storage. If the index is add-only, we can easily merge them even if they diverged. Each storage will have a copy of the index. The index on a storage is certainly always up to date about the secondary data on this storage (because we had access to the storage to update it). However, it might not always be up to date about the secondary data of the other storages. But eventually, the indexes will converge when the user backup to all storages.

On most scenario though, the main devices of the user, frequently updated, will have the most up-to-date indexes, and so we can give the user insights on the staging of the backups even if the copies are not connected yet. For instance, because we have the index of the hard drive on the laptop, we can know that the hard drive is up to date, even if the hard drive is not connected to the laptop. Finally, we can compare the only the index with the local device to know what is up to date and what is not.
