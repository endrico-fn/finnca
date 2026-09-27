class PrivacyState {
  enabled = $state(false);
  toggle() {
    this.enabled = !this.enabled;
  }
}
export const privacyState = new PrivacyState();
