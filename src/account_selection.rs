// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::model::ProviderId;

pub fn select_account_after_login(config: &mut Config, provider: ProviderId, account_id: String) {
    let ids = config.selected_account_ids_mut(provider);
    ids.clear();
    ids.push(account_id);
}
