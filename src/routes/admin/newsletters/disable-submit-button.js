const button = document.getElementById('submitButton');
const form = document.getElementById('publishForm');

button.addEventListener('click', () => {
  if (form.checkValidity()) {
    // Prevent button spamming while submitting the form
    button.setAttribute('disabled', 'true');
    form.submit();
  }
});
