export const initialDiagram = `flowchart TD
  Start([Open Mermaid Live])
  Edit[Write diagram syntax]
  Preview[Inspect live preview]
  Export{Ready to share?}
  Save[Save diagram]

  Start --> Edit --> Preview --> Export
  Export -- Yes --> Save
  Export -- Keep editing --> Edit
`;
