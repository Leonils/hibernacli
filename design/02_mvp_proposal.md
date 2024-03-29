# Proposal number 2 : MVP

## Context

This project is launched in the context of the TDD course of CentraleSupélec and a functional version shall be completed by march 29th, 2024. This proposal explores the main features of the MVP.

## Key features

The MVP shall be able to:

- Open a configuration file global to the device, defining :

  - Backup requirements (named classes of backups specifying a target number of copy, and a minimum security level)
  - Available storages (named devices with a type, and required information to connect to them)

- We shall support storage of type local (a mounted drive) for now.

- The user shall be able to categorize all files on his device between "ignored" (saved in the configuration file), "not categorized" (no info yet), and "to backup" (saved in a configuration file local to the project).

- The user shall be able to select a project and specify the backup requirements for this project.

- The user shall be able to select an available storage matching the requirements of the project and backup the project to this storage.

- The user shall be able to see the state of his backups (ignored, not categorized, backed up, up to date, outdated) and the progress of his categorization and backups.
