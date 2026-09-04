//! Integration tests for reporting failed assertions with their error message.

use miden_protocol::account::{Account, AccountComponent, AccountComponentMetadata};
use miden_protocol::utils::serde::{Deserializable, Serializable};
use miden_standards::code_builder::CodeBuilder;
use miden_testing::{Auth, MockChain};

/// An error message that exists only in this test, i.e. outside of the MASM sources of this repo.
const ERROR_MESSAGE: &str = "the caller is not allowed to poke this account";

const COMPONENT_PATH: &str = "test::poke";

/// Tests that an assertion of a user-defined account component renders its error message, even when
/// the account was serialized and thus lost the source-level debug info of its code.
#[tokio::test]
async fn user_component_assertion_renders_its_message() -> anyhow::Result<()> {
    let component_code = format!(
        r#"
        @account_procedure
        pub proc poke
            push.0 assert.err="{ERROR_MESSAGE}"
        end
        "#
    );

    let component_package =
        CodeBuilder::default().compile_component_code(COMPONENT_PATH, &component_code)?;
    let component = AccountComponent::new(
        component_package.clone(),
        vec![],
        AccountComponentMetadata::mock(COMPONENT_PATH),
    )?;

    let mut builder = MockChain::builder();
    let account = builder.add_existing_account_from_components(Auth::IncrNonce, [component])?;
    let account = Account::read_from_bytes(&account.to_bytes())?;
    builder.add_account(account.clone())?;
    let mock_chain = builder.build()?;

    let tx_script = CodeBuilder::default()
        .with_dynamically_linked_package(&component_package)?
        .compile_tx_script(format!(
            r#"
            use {COMPONENT_PATH} as poke

            @transaction_script
            pub proc main
                call.poke::poke
            end
            "#
        ))?;

    let Err(error) = mock_chain
        .build_transaction(account.id())
        .tx_script(tx_script)
        .build()?
        .execute()
        .await
    else {
        anyhow::bail!("the component assertion should fail");
    };

    let rendered = error.to_string();
    assert!(
        rendered.contains(ERROR_MESSAGE),
        "rendered error should contain the component's error message, but was: {rendered}"
    );

    Ok(())
}
