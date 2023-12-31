use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;
use std::time::Duration;

use chrono::NaiveDate;
use log::error;
use reqwest::blocking::Client;
use uuid::Uuid;

use crate::data::auth::{Auth, AuthType};
use crate::data::central_request_data::{CentralRequestDataList, CentralRequestItem};
use crate::data::collections::{Collection, CollectionFolder, Collections};
use crate::data::cookies_manager::{Cookie, CookiesManager};
use crate::data::environment::{Environment, EnvironmentConfig, EnvironmentItemValue};
use crate::data::history::{DateGroupHistoryList, HistoryDataList};
use crate::data::http::HttpRecord;
use crate::script::script::ScriptScope;
use crate::utils;

#[derive(Default, Clone, Debug)]
pub struct WorkspaceData {
    pub workspace_name: String,
    cookies_manager: RefCell<CookiesManager>,
    central_request_data_list: RefCell<CentralRequestDataList>,
    history_data_list: RefCell<HistoryDataList>,
    environment: RefCell<Environment>,
    collections: RefCell<Collections>,
    client: Option<Client>,
}

impl WorkspaceData {
    pub fn build_http_client(&mut self) -> Client {
        match &self.client {
            None => {
                let client = Client::builder()
                    .cookie_provider(self.cookies_manager.borrow().cookie_store.clone())
                    .trust_dns(true)
                    .tcp_nodelay(true)
                    .timeout(Duration::from_secs(60))
                    .build()
                    .unwrap_or_default();
                self.client = Some(client.clone());
                client
            }
            Some(client) => client.clone(),
        }
    }
}

//collections
impl WorkspaceData {
    pub fn get_collection(&self, option_path: Option<String>) -> Option<Collection> {
        let path = option_path?;
        let collection_name = path.splitn(2, "/").next()?;
        self.collections.borrow().data.get(collection_name).cloned()
    }

    pub fn get_folder_with_path(
        &self,
        path: String,
    ) -> (String, Option<Rc<RefCell<CollectionFolder>>>) {
        self.collections.borrow().get_folder_with_path(path)
    }

    pub fn collection_insert_http_record(
        &self,
        folder: Rc<RefCell<CollectionFolder>>,
        record: HttpRecord,
    ) {
        self.collections
            .borrow_mut()
            .insert_http_record(folder, record)
    }
    pub fn collection_remove_http_record(
        &self,
        folder: Rc<RefCell<CollectionFolder>>,
        collection_name: String,
    ) {
        self.collections
            .borrow_mut()
            .remove_http_record(folder, collection_name)
    }

    pub fn collection_insert_folder(
        &self,
        parent_folder: Rc<RefCell<CollectionFolder>>,
        folder: Rc<RefCell<CollectionFolder>>,
    ) {
        self.collections
            .borrow_mut()
            .insert_folder(parent_folder, folder)
    }

    pub fn get_collection_auth(&self, path: String) -> Auth {
        self.collections.borrow().get_auth(path)
    }

    pub fn get_collection_names(&self) -> HashSet<String> {
        self.collections
            .borrow()
            .get_data()
            .iter()
            .map(|(k, _)| k.to_string())
            .collect()
    }
    pub fn add_collection(&self, collection: Collection) {
        self.collections.borrow_mut().insert_collection(collection)
    }
    pub fn import_collection(&self, mut collection: Collection) -> String {
        let new_name = utils::build_copy_name(
            collection.folder.borrow().name.clone(),
            self.get_collection_names(),
        );
        collection.folder.borrow_mut().name = new_name.clone();
        self.add_collection(collection.clone());
        new_name
    }
    pub fn remove_collection(&self, collection_name: String) {
        self.collections
            .borrow_mut()
            .remove_collection(collection_name)
    }

    pub fn remove_folder(&self, parent_folder: Rc<RefCell<CollectionFolder>>, name: String) {
        self.collections
            .borrow_mut()
            .remove_folder(parent_folder, name)
    }

    pub fn get_collections(&self) -> BTreeMap<String, Collection> {
        self.collections.borrow().data.clone()
    }

    pub fn update_collection_folder(&self, folder: Rc<RefCell<CollectionFolder>>) {
        self.collections.borrow().update_folder(folder)
    }
}

//env
impl WorkspaceData {
    pub fn get_build_envs(
        &self,
        collection: Option<Collection>,
    ) -> BTreeMap<String, EnvironmentItemValue> {
        self.environment.borrow().get_variable_hash_map(collection)
    }
    pub fn get_env_select(&self) -> Option<String> {
        self.environment.borrow().select()
    }
    pub fn set_env_select(&self, select: Option<String>) {
        self.environment.borrow_mut().set_select(select)
    }

    pub fn get_env_configs(&self) -> BTreeMap<String, EnvironmentConfig> {
        self.environment.borrow().get_data()
    }

    pub fn get_env(&self, key: String) -> Option<EnvironmentConfig> {
        self.environment.borrow().get(key)
    }

    pub fn add_env(&self, key: String, value: EnvironmentConfig) {
        self.environment.borrow_mut().insert(key, value)
    }

    pub fn remove_env(&self, key: String) {
        self.environment.borrow_mut().remove(key)
    }
}

// history
impl WorkspaceData {
    pub fn get_history_group(&self) -> BTreeMap<NaiveDate, DateGroupHistoryList> {
        self.history_data_list.borrow().get_group().clone()
    }
    pub fn history_record(&self, rest: HttpRecord) {
        self.history_data_list.borrow_mut().record(rest);
    }
}
// cookie
impl WorkspaceData {
    pub fn get_url_cookies(&self, url: String) -> BTreeMap<String, Cookie> {
        self.cookies_manager.borrow().get_url_cookies(url)
    }
    pub fn save_cookies(&self) {
        self.cookies_manager.borrow().save()
    }
    pub fn cookies_contain_domain(&self, domain: String) -> bool {
        self.cookies_manager.borrow().contain_domain(domain)
    }
    pub fn cookies_contain_domain_key(&self, domain: String, key: String) -> bool {
        self.cookies_manager
            .borrow()
            .contain_domain_key(domain, key)
    }
    pub fn add_domain_cookies(&self, cookie: Cookie) -> Result<(), String> {
        self.cookies_manager.borrow_mut().add_domain_cookies(cookie)
    }
    pub fn get_cookie_domains(&self) -> Vec<String> {
        self.cookies_manager.borrow().get_cookie_domains()
    }

    pub fn get_domain_cookies(&self, domain: String) -> Option<BTreeMap<String, Cookie>> {
        self.cookies_manager.borrow().get_domain_cookies(domain)
    }
    pub fn remove_cookie_domain(&self, domain: String) {
        self.cookies_manager.borrow_mut().remove_domain(domain)
    }
    pub fn remove_cookie_domain_path_name(&mut self, domain: String, path: String, name: String) {
        self.cookies_manager
            .borrow_mut()
            .remove_domain_path_name(domain, path, name)
    }

    pub fn update_domain_cookies(
        &self,
        cookie: Cookie,
        domain: String,
        name: String,
    ) -> Result<(), String> {
        self.cookies_manager
            .borrow_mut()
            .update_domain_cookies(cookie, domain, name)
    }
}

// crt
impl WorkspaceData {
    pub fn save_crt(
        &mut self,
        crt_id: String,
        collection_path: String,
        modify_http_record: impl FnOnce(&mut HttpRecord),
    ) {
        let mut new_name_option = None;
        self.central_request_data_list
            .borrow_mut()
            .data_map
            .get_mut(crt_id.as_str())
            .map(|crt| {
                let (_, cf_option) = self.get_folder_with_path(collection_path.clone());
                cf_option.map(|cf| {
                    let mut http_record = crt.rest.clone();
                    modify_http_record(&mut http_record);
                    new_name_option = Some(http_record.name.clone());
                    self.collection_insert_http_record(cf.clone(), http_record);
                    crt.set_baseline();
                });
            });
        new_name_option.map(|new_name| {
            self.central_request_data_list
                .borrow_mut()
                .update_old_id_to_new(crt_id, collection_path.clone(), new_name.clone());
        });
    }

    pub fn must_get_mut_crt(&self, id: String, call: impl FnOnce(&mut CentralRequestItem)) {
        match self
            .central_request_data_list
            .borrow_mut()
            .data_map
            .get_mut(id.as_str())
        {
            None => {
                error!("get crt:{} error", id)
            }
            Some(crt) => call(crt),
        }
    }
    pub fn get_crt_envs(&self, id: String) -> BTreeMap<String, EnvironmentItemValue> {
        let crt = self.must_get_crt(id);
        self.get_build_envs(self.get_collection(crt.collection_path.clone()))
    }

    pub fn get_crt_parent_auth(&self, id: String) -> Auth {
        let crt = self.must_get_crt(id);
        match &crt.collection_path {
            None => Auth {
                auth_type: AuthType::NoAuth,
                basic_username: "".to_string(),
                basic_password: "".to_string(),
                bearer_token: "".to_string(),
            },
            Some(collection_path) => self.get_collection_auth(collection_path.clone()),
        }
    }

    pub fn get_crt_parent_scripts(&self, id: String) -> (Vec<ScriptScope>, Vec<ScriptScope>) {
        let crt = self.must_get_crt(id);
        let mut pre_request_script_scopes = Vec::new();
        let mut test_script_scopes = Vec::new();
        match &crt.collection_path {
            None => {}
            Some(collection_path) => {
                (pre_request_script_scopes, test_script_scopes) = self
                    .collections
                    .borrow()
                    .get_path_scripts(collection_path.clone())
            }
        }
        (pre_request_script_scopes, test_script_scopes)
    }

    pub fn get_crt_select_id(&self) -> Option<String> {
        self.central_request_data_list.borrow().select_id.clone()
    }

    pub fn set_crt_select_id(&self, select_id: Option<String>) {
        self.central_request_data_list.borrow_mut().select_id = select_id;
    }

    pub fn get_crt_id_list(&self) -> Vec<String> {
        self.central_request_data_list.borrow().data_list.clone()
    }
    pub fn get_crt_id_set(&self) -> HashSet<String> {
        let mut hashset = HashSet::new();
        for id in self.get_crt_id_list() {
            hashset.insert(id);
        }
        hashset
    }

    pub fn get_crt_cloned(&self, id: String) -> Option<CentralRequestItem> {
        self.central_request_data_list
            .borrow()
            .data_map
            .get(id.as_str())
            .cloned()
    }
    pub fn must_get_crt(&self, id: String) -> CentralRequestItem {
        self.central_request_data_list
            .borrow()
            .data_map
            .get(id.as_str())
            .cloned()
            .unwrap_or_else(|| {
                error!("get crt {} error", id);
                CentralRequestItem::default()
            })
    }
    pub fn auto_save_crd(&self) {
        self.central_request_data_list.borrow().auto_save();
    }

    pub fn add_crt(&self, crt: CentralRequestItem) {
        self.central_request_data_list.borrow_mut().add_crt(crt);
    }

    pub fn close_all_crt(&self) {
        self.central_request_data_list.borrow_mut().clear();
        self.set_crt_select_id(None);
    }
    pub fn close_other_crt(&self, id: String) {
        let retain = self.must_get_crt(id.clone()).clone();
        self.central_request_data_list.borrow_mut().clear();
        self.add_crt(retain);
        self.set_crt_select_id(Some(id.clone()));
    }

    pub fn close_crt(&self, id: String) {
        self.central_request_data_list
            .borrow_mut()
            .remove(id.clone());
        if let Some(select_id) = self.get_crt_select_id() {
            if select_id == id {
                self.set_crt_select_id(None);
            }
        }
    }

    pub fn duplicate_crt(&self, id: String) {
        let mut duplicate = self.must_get_crt(id).clone();
        duplicate.id = Uuid::new_v4().to_string();
        duplicate.collection_path = None;
        self.add_crt(duplicate);
    }

    pub fn add_new_crt(&self) {
        self.central_request_data_list.borrow_mut().add_new();
    }

    pub fn contains_crt_id(&self, crt_id: String) -> bool {
        self.central_request_data_list
            .borrow()
            .data_map
            .contains_key(crt_id.as_str())
    }

    pub fn update_crt_old_name_to_new_name(
        &self,
        path: String,
        old_name: String,
        new_name: String,
    ) {
        self.central_request_data_list
            .borrow_mut()
            .update_old_name_to_new_name(path, old_name, new_name);
    }
}

impl WorkspaceData {
    pub fn load_all(&mut self, workspace: String) {
        self.central_request_data_list
            .borrow_mut()
            .load_all(workspace.clone());
        self.history_data_list
            .borrow_mut()
            .load_all(workspace.clone());
        self.environment.borrow_mut().load_all(workspace.clone());
        self.collections.borrow_mut().load_all(workspace.clone());
        self.cookies_manager
            .borrow_mut()
            .load_all(workspace.clone())
    }
    pub fn reload_data(&mut self, workspace: String) {
        self.history_data_list
            .borrow_mut()
            .load_all(workspace.clone());
        self.environment.borrow_mut().load_all(workspace.clone());
        self.collections.borrow_mut().load_all(workspace.clone());
        self.cookies_manager
            .borrow_mut()
            .load_all(workspace.clone())
    }
}
