# Proposal number 2 : MVP

## Context

This project is launched in the context of the TDD course of CentraleSupélec
and a functional version shall be completed by march 29th, 2024. This proposal
explores the main features of the MVP.

## Key hypothesis

For the first prototype, we may assume the followings, keeping in mind they will
become false at one point in time.

- As a user, I have a single primary device, with only primary data, and I
  connect to it secondary devices, with only secondary data. Let's call this
  device the **primary device**

- As a user, my only secondary devices are devices physically connected to the
  computer (USB keys, External hard drives, secondary partitions, ...)

## Key features

1. As a user, I have a **global configuration file** on my _primary device_, so the
  following settings are saved:

    - Backup requirements (named classes of backups specifying a target number of
      copy, and a minimum security level)
    - Available storages (named devices with a type, a provided security, and
      required information to connect to them)

2. As a user I can navigate files on my device and identify "projects" : set of
  files all related together

3. As a user, I can categorize projects on his device between the states:

    - **ignored** (explicit, saved in the configuration file),
    - **not categorized** (implicit, no info saved neither in local configuration
      file, nor in local one),
    - **to backup** (saved in a configuration file local to the project, so moving
      it do not break the system).

4. As a user, I can navigate projects and see their state. When a target backup
  requirement class is specified, I can see what is the status of the backup
  process: does it is actually backuped? On which device? what is the status of
  each backup? Are they up to date?

5. As a user, I can specify or upgrade the requirement class of a project. Then,
  I can choose among available storages to fullfill these requirement. I'm guided
  through this process

6. As a user, I can run a backup from the primary to a given secondary, and it will
  backup all projects configured to be backed up on this secondary that are not
  up to date

7. As a user, I can explore the content of a secondary drive, and restore a project
  by downloading a zip with all of its content on my primary device.

