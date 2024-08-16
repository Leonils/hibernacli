use crate::core::device::Device;

use super::super::GlobalConfig;

impl GlobalConfig {
    pub fn get_device_by_name(&self, name: &str) -> Option<&Box<dyn Device>> {
        self.devices.iter().find(|d| d.get_name() == name)
    }

    pub fn add_device(&mut self, device: Box<dyn Device>) -> Result<(), String> {
        if self.get_device_by_name(&device.get_name()).is_some() {
            return Err(format!(
                "Device with name {} already exists",
                device.get_name()
            ));
        }

        self.devices.push(device);
        Ok(())
    }

    pub fn remove_device(&mut self, name: &str) -> Result<(), String> {
        let index = self
            .devices
            .iter()
            .position(|d| d.get_name() == name)
            .ok_or_else(|| "Device not found".to_string())?;

        self.devices.remove(index);
        Ok(())
    }

    pub fn get_devices(self) -> Vec<Box<dyn Device>> {
        self.devices
    }

    pub fn get_devices_iter(&self) -> impl Iterator<Item = &Box<dyn Device>> {
        self.devices.iter()
    }
}

#[cfg(test)]
mod tests {

    use crate::core::{
        test_utils::mocks::{MockDevice, MockDeviceFactory},
        DeviceFactory,
    };

    use super::GlobalConfig;

    #[test]
    fn when_adding_device_to_global_config_it_shall_add_it() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let device = MockDeviceFactory
            .build_from_toml_table("MyPersonalDevice", &toml::Table::new())
            .unwrap();

        global_config.add_device(device).unwrap();
        assert_eq!(global_config.devices.len(), 1);
        assert_eq!(global_config.devices[0].get_name(), "MyPersonalDevice");
    }

    #[test]
    fn when_adding_device_to_global_config_if_device_already_exists_it_shall_return_error() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let device = MockDeviceFactory
            .build_from_toml_table("MyPersonalDevice", &toml::Table::new())
            .unwrap();

        let device2 = MockDeviceFactory
            .build_from_toml_table("MyPersonalDevice", &toml::Table::new())
            .unwrap();

        let result = global_config.add_device(device);
        assert!(result.is_ok());

        let result = global_config.add_device(device2);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Device with name MyPersonalDevice already exists"
        );

        assert_eq!(global_config.devices.len(), 1);
    }

    #[test]
    fn when_removing_device_from_global_config_it_shall_remove_it() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let device1 = MockDevice::new("MyPersonalDevice");
        let device2 = MockDevice::new("MySecondPersonalDevice");

        global_config.add_device(Box::new(device1)).unwrap();
        global_config.add_device(Box::new(device2)).unwrap();
        assert_eq!(global_config.devices.len(), 2);

        global_config.remove_device("MyPersonalDevice").unwrap();
        assert_eq!(global_config.devices.len(), 1);
        assert_eq!(
            global_config.devices[0].get_name(),
            "MySecondPersonalDevice"
        );
    }

    #[test]
    fn when_removing_non_existant_device_it_shall_return_error() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };
        let result = global_config.remove_device("NonExistantDevice");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Device not found");
    }
}
