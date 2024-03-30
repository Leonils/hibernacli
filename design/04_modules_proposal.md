# Proposal number 4: main modules

Modules are the core elements of a rust crate. If we have to do a minimal amount
of upfront design, identifying responsibilities of modules seems the thing to do

First, the **Business logic** modules, defining the interfaces and the actual 
operations

```rs
// Generic business models used by everyone (ports)
mod models {
    mod backup_requirement;
    mod secondary_device;
    mod primary_device;
}

// Configuration parsing
mod config {
    mod global_config;
    mod project_config;
}

// Actual business operations
mod operations {
    mod backup;
    mod restore;
    mod device_scan;
}
```

Then the actual implementations of the primary and secondary devices : the 
**Server side** modules:

```rs
// concrete implementations of storage (local, remote, ...)
mod devices {
    mod local_file_storage;
    // mod s3_bucket (for the future)
}
```

Finally the interactions with the user : the **User side** modules:

```rs
// Interactions with user
mod cli {


}
```

Following principles of the hexagonal architecture, the interfaces are defined
business side, and the user side and server side depends on the business side, 
not the opposite. The business side defines ports and business logic depending 
on them, and the user and server side define addapter implementing ports. 
