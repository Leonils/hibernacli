# Proposal number 6 : User stories

In order to define the features required to the backup tool, we need user stories that cover use cases for the tool. These should be the point of focus of testing and development. They will represent **what** the code does, in order to create the **how** as a consequence. This proposal attempts to describe them.

One may find this file to be incomplete, as it only represents the feature of a Minimum Viable Product. A backup over the network is not included, as well as an automated way to backup data, or a description of differences between backup. Those may be included in further versions.

## Global configuration

As a user **using the CLI**, I want to be able to:

- `(1)` after plugging in my USB stick, HDD or SSD, create a new entry for it by providing all relevant information (name, type, location).
- `(2)` edit and delete the properties of any support that has previously been registered.

## Project configuration

`(3)` As a user, I want to be able to initialize or edit a backup strategy for a specific repository by providing all the required configuration of that strategy : physical replication requirements, number of replication requirements, amount of security of said replications. For this repository I can choose to provide with a whitelist and/or a blacklist of security levels for each file/folder that should be included in the backup. The state of the backup should follow the following rules:

- backup statuses are **inherited by default**. Without any instruction for a specific file or folder (through a blacklist/whitelist), the status of a file/folder is the one of its parent repository. If the parent repository is not part of the backup project, the status is determined by the configuration of the project. If the configuration cannot determine if the file/folder is to be backed up, it is `not categorized`
- backup status are **ordered** according to the following rule: **Not Categorized** is less restrictive that **Ignored** which is less restrictive than **Backed up**. A file/folder cannot have a more restrictive status than its parent repository. For example, a `misc` folder that is `ignored` restricts any file/folder inside it to be `ignored` or `not categorized`.

**Note:** As this proposal is for an MVP only, the configuration editing step may be done by editing a file.

## Backup process

`(4)` As a user, I want to be able to plug my storage device, get the properties of the last backup and start a new backup. The backup process should follow the rules of the project configuration as specified above.

## Data recovery

`(5)` As a user, I want to be able to navigate the filesystem of my external storage device and find for each distinct project a subtree of the project's filesystem. The subtree should be the snapshot of the project at that time of last backup, that includes only the files/folder that have the `backed up` status. The rules mentionned before ensure this subtree is connex.

**Note**: Again, the data recovery process could be done through the CLI in a second step.
