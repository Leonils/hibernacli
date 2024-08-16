use super::super::GlobalConfig;
use crate::models::project::Project;

impl GlobalConfig {
    pub fn get_project_by_name(&self, name: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.get_name() == name)
    }

    pub fn add_project(&mut self, project: Project) -> Result<(), String> {
        if self.get_project_by_name(&project.get_name()).is_some() {
            return Err(format!(
                "Project with name {} already exists",
                project.get_name()
            ));
        }
        if self.get_project_by_path(&project.get_location()).is_some() {
            return Err(format!(
                "Project with path {} already exists",
                project.get_location()
            ));
        }
        self.projects.push(project);
        Ok(())
    }

    pub fn remove_project(&mut self, name: &str) -> Result<(), String> {
        let index = self
            .projects
            .iter()
            .position(|p| p.get_name() == name)
            .ok_or_else(|| "Project not found".to_string())?;

        self.projects.remove(index);
        Ok(())
    }

    pub fn get_projects(self) -> Vec<Project> {
        self.projects
    }

    pub fn get_projects_iter(&self) -> impl Iterator<Item = &Project> {
        self.projects.iter()
    }

    fn get_project_by_path(&self, path: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.get_location() == path)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn when_we_add_a_project_to_the_config_it_shall_be_visible() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let project = Project::new("MyProject".to_string(), "/tmp".to_string(), None);

        global_config.add_project(project).unwrap();
        assert_eq!(global_config.projects.len(), 1);
        assert_eq!(global_config.projects[0].get_name(), "MyProject");
        assert_eq!(global_config.projects[0].get_location(), "/tmp");
    }

    #[test]
    fn when_we_add_multiple_projects_to_the_config_it_shall_be_visible() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let project = Project::new("MyProject".to_string(), "/tmp".to_string(), None);
        let project2 = Project::new("MySecondProject".to_string(), "/root".to_string(), None);

        global_config.add_project(project).unwrap();
        global_config.add_project(project2).unwrap();
        assert_eq!(global_config.projects.len(), 2);
        assert_eq!(global_config.projects[0].get_name(), "MyProject");
        assert_eq!(global_config.projects[0].get_location(), "/tmp");
        assert_eq!(global_config.projects[1].get_name(), "MySecondProject");
        assert_eq!(global_config.projects[1].get_location(), "/root");
    }

    #[test]
    fn when_we_add_a_project_with_same_name_it_should_return_error() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let project = Project::new("MyProject".to_string(), "/tmp".to_string(), None);
        let project2 = Project::new("MyProject".to_string(), "/tmp".to_string(), None);

        global_config.add_project(project).unwrap();
        let result = global_config.add_project(project2);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Project with name MyProject already exists"
        );
    }

    #[test]
    fn when_we_add_a_project_with_the_same_path_it_should_return_error() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let project = Project::new("MyProject".to_string(), "/tmp".to_string(), None);
        let project2 = Project::new("MySecondProject".to_string(), "/tmp".to_string(), None);

        global_config.add_project(project).unwrap();
        let result = global_config.add_project(project2);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Project with path /tmp already exists"
        );
    }

    #[test]
    fn when_deleting_project_it_shall_be_removed() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let project = Project::new("MyProject".to_string(), "/tmp".to_string(), None);
        let project2 = Project::new("MySecondProject".to_string(), "/root".to_string(), None);

        global_config.add_project(project).unwrap();
        global_config.add_project(project2).unwrap();
        assert_eq!(global_config.projects.len(), 2);

        global_config.remove_project("MyProject").unwrap();
        assert_eq!(global_config.projects.len(), 1);
        assert_eq!(global_config.projects[0].get_name(), "MySecondProject");
    }

    #[test]
    fn when_deleting_not_registered_project_it_shoud_throw_an_error() {
        let mut global_config = GlobalConfig {
            devices: vec![],
            projects: vec![],
        };

        let result = global_config.remove_project("MyProject");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Project not found");
    }
}
