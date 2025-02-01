/**
 * A collection of types to help define the data associated with custom events.
 */

/**
 * Detail for the "SolutionsUpdated" event.
 */
export interface SolutionsUpdatedDetail {
  requestId: number;
  isFinalResponse: boolean;
  solutions: string[][];
}

// Extend the global DocumentEventMap for custom events
declare global {
  interface DocumentEventMap {
    "SolutionsUpdated": CustomEvent<SolutionsUpdatedDetail>;
  }
}
