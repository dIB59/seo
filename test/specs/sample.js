describe('Hello Tauri', () => {
    it('should get response from Rust code', async () => {
        // Find the input field and type in a name
        const input = $('#greet-input');
        await input.setValue('John');

        // Submit the form
        const button = $('button[type="submit"]');
        await button.click();

        // Wait for the response message to appear
        const greeting = $('#greet-response');
        await greeting.waitUntil(async function () {
            return (await this.getText()).includes('Hello, John!')
        }, {
            timeout: 5000,
            timeoutMsg: 'expected text to be available after 5s'
        });

        console.log(await greeting.getText())

        // Verify that the greeting message contains the entered name
        expect(await greeting.getText()).toHaveText('Hello, John!');
    })
})