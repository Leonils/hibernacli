# Proposal number 6 : User stories

In order to define the features required to the backup tool, we need user stories that cover use cases for the tool. These should be the point of focus of testing and development. These represent the flow of the user through the tool. It should mention or imply the features that are explicited in proposal 2.

One may find this file to be incomplete, as it only represents the feature of a Minimum Viable Product. A backup over the network is not included, as well as an automated way to backup data, or a description of differences between backup. Those may be included in further versions.

## User Story 1 : Project configuration

- **Initialize a device** : I want to plug in a physical storage device. From the CLI, I can choose the device to register it in the backup tool. The CLI should guide me through the process of initializing the device.
- **Initialize a project** : for project at path `/path/to/project/`, I want to initialize a config file that represents the configuration of the backup. This configuration file should be editable by me.

## User Story 2 : Backup process

From a device that has been initialized in which a project has been configured.

- **Backup my projects**: From the CLI, I want to run a command that will synchronize all available storage devices on all configured projects.

## User Story 3 : Device management

- **List devices** : From the CLI, I want to list all the devices that have been initialized.
- **Remove a device** : From the CLI, I want to remove a device that has been initialized.

## User Story 4 : Project management

- **List projects** : From the CLI, I want to list all the projects that have been configured.
- **Remove a project** : From the CLI, I want to remove a project that has been configured.
