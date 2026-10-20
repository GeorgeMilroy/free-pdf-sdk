/** Free PDF SDK (phase 1 skeleton: mounts the viewer page in an iframe). */
export async function init(element: HTMLElement, options: { baseUrl: string }): Promise<HTMLIFrameElement> {
  const frame = document.createElement("iframe");
  frame.src = new URL("viewer/index.html", new URL(options.baseUrl, location.href)).href;
  frame.style.cssText = "width:100%;height:100%;border:0;display:block";
  element.appendChild(frame);
  return frame;
}
