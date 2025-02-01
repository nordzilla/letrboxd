/**
 * A11yAnnouncer is a simple utility class that creates a hidden
 * live region on the document body and provides an announce method
 * for accessible status updates.
 */
export class A11yAnnouncer {
  constructor() {
    this.announcer = document.createElement("div");
    this.announcer.setAttribute("aria-live", "polite");
    this.announcer.style.position = "absolute";
    this.announcer.style.left = "-9999px";
    this.announcer.textContent = "";
    document.body.appendChild(this.announcer);
  }
  
  /**
   * Announces a message to assistive technology. We clear first to
   * ensure the screen reader recognizes a change and then apply a
   * slight delay before setting the actual content.
   *
   * @param {string} message - The message to be announced.
   */
  announce(message) {
    this.announcer.textContent = "";
    setTimeout(() => {
      this.announcer.textContent = message;
    }, 100);
  }
  
  /**
   * Removes the live region from the DOM and sets the announcer element to null.
   * Call this when the announcer is no longer needed.
   */
  disconnect() {
    if (this.announcer && this.announcer.parentNode) {
      this.announcer.parentNode.removeChild(this.announcer);
    }
    this.announcer = null;
  }
}
