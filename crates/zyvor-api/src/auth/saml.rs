// SPDX-License-Identifier: Apache-2.0

use super::types::SamlSettings;
use crate::config::Config;

pub fn sp_metadata_xml(config: &Config, saml: &SamlSettings) -> String {
    let entity_id = if saml.entity_id.is_empty() {
        format!("{}/api/v1/auth/saml/metadata", config.public_base_url.trim_end_matches('/'))
    } else {
        saml.entity_id.clone()
    };
    let acs = format!(
        "{}/api/v1/auth/saml/acs",
        config.public_base_url.trim_end_matches('/')
    );
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata" entityID="{entity_id}">
  <SPSSODescriptor AuthnRequestsSigned="false" WantAssertionsSigned="true" protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <NameIDFormat>{name_id_format}</NameIDFormat>
    <AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="{acs}" index="1"/>
  </SPSSODescriptor>
</EntityDescriptor>"#,
        entity_id = xml_escape(&entity_id),
        name_id_format = xml_escape(if saml.name_id_format.is_empty() {
            "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"
        } else {
            &saml.name_id_format
        }),
        acs = xml_escape(&acs),
    )
}

fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
