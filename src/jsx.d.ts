type EventHandler<E extends Event = Event> = (event: E) => void;

declare namespace JSX {
  interface IntrinsicElements {
    // Document metadata
    html: HTMLAttributes<HTMLHtmlElement>;
    head: HTMLAttributes<HTMLHeadElement>;
    title: HTMLAttributes<HTMLTitleElement>;
    base: HTMLAttributes<HTMLBaseElement>;
    link: HTMLAttributes<HTMLLinkElement>;
    meta: HTMLAttributes<HTMLMetaElement>;
    style: HTMLAttributes<HTMLStyleElement>;

    // Sectioning root
    body: HTMLAttributes<HTMLBodyElement>;

    // Content sectioning
    address: HTMLAttributes<HTMLElement>;
    article: HTMLAttributes<HTMLElement>;
    aside: HTMLAttributes<HTMLElement>;
    footer: HTMLAttributes<HTMLElement>;
    header: HTMLAttributes<HTMLElement>;
    h1: HTMLAttributes<HTMLHeadingElement>;
    h2: HTMLAttributes<HTMLHeadingElement>;
    h3: HTMLAttributes<HTMLHeadingElement>;
    h4: HTMLAttributes<HTMLHeadingElement>;
    h5: HTMLAttributes<HTMLHeadingElement>;
    h6: HTMLAttributes<HTMLHeadingElement>;
    main: HTMLAttributes<HTMLElement>;
    nav: HTMLAttributes<HTMLElement>;
    section: HTMLAttributes<HTMLElement>;

    // Text content
    blockquote: HTMLAttributes<HTMLElement>;
    dd: HTMLAttributes<HTMLElement>;
    div: HTMLAttributes<HTMLDivElement>;
    dl: HTMLAttributes<HTMLDListElement>;
    dt: HTMLAttributes<HTMLElement>;
    figcaption: HTMLAttributes<HTMLElement>;
    figure: HTMLAttributes<HTMLElement>;
    hr: HTMLAttributes<HTMLHRElement>;
    li: HTMLAttributes<HTMLLIElement>;
    ol: HTMLAttributes<HTMLOListElement>;
    p: HTMLAttributes<HTMLParagraphElement>;
    pre: HTMLAttributes<HTMLPreElement>;
    ul: HTMLAttributes<HTMLUListElement>;

    // Inline text semantics
    a: AnchorHTMLAttributes<HTMLAnchorElement>;
    abbr: HTMLAttributes<HTMLElement>;
    b: HTMLAttributes<HTMLElement>;
    bdi: HTMLAttributes<HTMLElement>;
    bdo: HTMLAttributes<HTMLElement>;
    br: HTMLAttributes<HTMLBRElement>;
    cite: HTMLAttributes<HTMLElement>;
    code: HTMLAttributes<HTMLElement>;
    data: HTMLAttributes<HTMLDataElement>;
    dfn: HTMLAttributes<HTMLElement>;
    em: HTMLAttributes<HTMLElement>;
    i: HTMLAttributes<HTMLElement>;
    kbd: HTMLAttributes<HTMLElement>;
    mark: HTMLAttributes<HTMLElement>;
    q: HTMLAttributes<HTMLQuoteElement>;
    rp: HTMLAttributes<HTMLElement>;
    rt: HTMLAttributes<HTMLElement>;
    ruby: HTMLAttributes<HTMLElement>;
    s: HTMLAttributes<HTMLElement>;
    samp: HTMLAttributes<HTMLElement>;
    small: HTMLAttributes<HTMLElement>;
    span: HTMLAttributes<HTMLSpanElement>;
    strong: HTMLAttributes<HTMLElement>;
    sub: HTMLAttributes<HTMLElement>;
    sup: HTMLAttributes<HTMLElement>;
    time: HTMLAttributes<HTMLTimeElement>;
    u: HTMLAttributes<HTMLElement>;
    var: HTMLAttributes<HTMLElement>;
    wbr: HTMLAttributes<HTMLElement>;

    // Image and multimedia
    area: AreaHTMLAttributes<HTMLAreaElement>;
    audio: AudioHTMLAttributes<HTMLAudioElement>;
    img: ImgHTMLAttributes<HTMLImageElement>;
    map: MapHTMLAttributes<HTMLMapElement>;
    track: TrackHTMLAttributes<HTMLTrackElement>;
    video: VideoHTMLAttributes<HTMLVideoElement>;

    // Embedded content
    embed: EmbedHTMLAttributes<HTMLEmbedElement>;
    iframe: IframeHTMLAttributes<HTMLIFrameElement>;
    object: ObjectHTMLAttributes<HTMLObjectElement>;
    param: ParamHTMLAttributes<HTMLParamElement>;
    picture: HTMLAttributes<HTMLElement>;
    source: SourceHTMLAttributes<HTMLSourceElement>;

    // Scripting
    canvas: CanvasHTMLAttributes<HTMLCanvasElement>;
    noscript: HTMLAttributes<HTMLElement>;
    script: ScriptHTMLAttributes<HTMLScriptElement>;

    // Demarcating edits
    del: DelHTMLAttributes<HTMLModElement>;
    ins: InsHTMLAttributes<HTMLModElement>;

    // Table content
    caption: HTMLAttributes<HTMLElement>;
    col: ColHTMLAttributes<HTMLTableColElement>;
    colgroup: ColgroupHTMLAttributes<HTMLTableColElement>;
    table: TableHTMLAttributes<HTMLTableElement>;
    tbody: HTMLAttributes<HTMLTableSectionElement>;
    td: TdHTMLAttributes<HTMLTableDataCellElement>;
    tfoot: HTMLAttributes<HTMLTableSectionElement>;
    th: ThHTMLAttributes<HTMLTableHeaderCellElement>;
    thead: HTMLAttributes<HTMLTableSectionElement>;
    tr: HTMLAttributes<HTMLTableRowElement>;

    // Forms
    button: ButtonHTMLAttributes<HTMLButtonElement>;
    datalist: HTMLAttributes<HTMLDataListElement>;
    fieldset: FieldsetHTMLAttributes<HTMLFieldSetElement>;
    form: FormHTMLAttributes<HTMLFormElement>;
    input: InputHTMLAttributes<HTMLInputElement>;
    label: LabelHTMLAttributes<HTMLLabelElement>;
    legend: HTMLAttributes<HTMLLegendElement>;
    meter: MeterHTMLAttributes<HTMLMeterElement>;
    optgroup: OptgroupHTMLAttributes<HTMLOptGroupElement>;
    option: OptionHTMLAttributes<HTMLOptionElement>;
    output: OutputHTMLAttributes<HTMLOutputElement>;
    progress: ProgressHTMLAttributes<HTMLProgressElement>;
    select: SelectHTMLAttributes<HTMLSelectElement>;
    textarea: TextareaHTMLAttributes<HTMLTextAreaElement>;

    // Interactive elements
    details: DetailsHTMLAttributes<HTMLDetailsElement>;
    dialog: HTMLAttributes<HTMLDialogElement>;
    menu: MenuHTMLAttributes<HTMLMenuElement>;
    summary: HTMLAttributes<HTMLElement>;

    // Web Components
    slot: HTMLAttributes<HTMLSlotElement>;
    template: HTMLAttributes<HTMLTemplateElement>;

    // Allow arbitrary custom elements
    [elemName: string]: unknown;
  }

  interface HTMLAttributes<_T> {
    id?: string;
    class?: string;
    style?: string | Partial<CSSStyleDeclaration>;
    title?: string;
    tabIndex?: number;
    role?: string;
    children?: unknown;

    // Events
    onClick?: EventHandler<MouseEvent>;
    onInput?: EventHandler<InputEvent>;
    onChange?: EventHandler<Event>;
    onFocus?: EventHandler<FocusEvent>;
    onBlur?: EventHandler<FocusEvent>;
    onKeyDown?: EventHandler<KeyboardEvent>;
    onKeyUp?: EventHandler<KeyboardEvent>;
    onKeyPress?: EventHandler<KeyboardEvent>;
    onMouseDown?: EventHandler<MouseEvent>;
    onMouseUp?: EventHandler<MouseEvent>;
    onMouseMove?: EventHandler<MouseEvent>;
    onMouseEnter?: EventHandler<MouseEvent>;
    onMouseLeave?: EventHandler<MouseEvent>;
    onSubmit?: EventHandler<Event>;
    // ...добавь другие события по необходимости

    [key: string]: unknown;
  }

  interface AnchorHTMLAttributes<T> extends HTMLAttributes<T> {
    href?: string;
    target?: string;
    rel?: string;
    download?: unknown;
    hreflang?: string;
    type?: string;
  }

  interface AreaHTMLAttributes<T> extends HTMLAttributes<T> {
    alt?: string;
    coords?: string;
    download?: unknown;
    href?: string;
    hreflang?: string;
    rel?: string;
    shape?: string;
    target?: string;
  }

  interface AudioHTMLAttributes<T> extends MediaHTMLAttributes<T> {}

  interface VideoHTMLAttributes<T> extends MediaHTMLAttributes<T> {
    height?: number | string;
    width?: number | string;
    poster?: string;
  }

  interface MediaHTMLAttributes<T> extends HTMLAttributes<T> {
    autoPlay?: boolean;
    controls?: boolean;
    loop?: boolean;
    muted?: boolean;
    preload?: string;
    src?: string;
  }

  interface ImgHTMLAttributes<T> extends HTMLAttributes<T> {
    alt?: string;
    src?: string;
    srcSet?: string;
    height?: number | string;
    width?: number | string;
    loading?: 'eager' | 'lazy';
    referrerPolicy?: string;
    decoding?: 'async' | 'auto' | 'sync';
    crossOrigin?: 'anonymous' | 'use-credentials' | '';
  }

  interface MapHTMLAttributes<T> extends HTMLAttributes<T> {
    name?: string;
  }

  interface TrackHTMLAttributes<T> extends HTMLAttributes<T> {
    default?: boolean;
    kind?: string;
    label?: string;
    src?: string;
    srcLang?: string;
  }

  interface SourceHTMLAttributes<T> extends HTMLAttributes<T> {
    src?: string;
    type?: string;
    srcSet?: string;
    media?: string;
    sizes?: string;
  }

  interface EmbedHTMLAttributes<T> extends HTMLAttributes<T> {
    height?: number | string;
    src?: string;
    type?: string;
    width?: number | string;
  }

  interface IframeHTMLAttributes<T> extends HTMLAttributes<T> {
    allow?: string;
    allowFullScreen?: boolean;
    height?: number | string;
    name?: string;
    referrerPolicy?: string;
    sandbox?: string;
    src?: string;
    srcDoc?: string;
    width?: number | string;
  }

  interface ObjectHTMLAttributes<T> extends HTMLAttributes<T> {
    data?: string;
    form?: string;
    height?: number | string;
    name?: string;
    type?: string;
    useMap?: string;
    width?: number | string;
  }

  interface ParamHTMLAttributes<T> extends HTMLAttributes<T> {
    name?: string;
    value?: string | string[] | number;
  }

  interface CanvasHTMLAttributes<T> extends HTMLAttributes<T> {
    height?: number | string;
    width?: number | string;
  }

  interface ScriptHTMLAttributes<T> extends HTMLAttributes<T> {
    async?: boolean;
    defer?: boolean;
    src?: string;
    type?: string;
    crossOrigin?: string;
    integrity?: string;
    noModule?: boolean;
    nonce?: string;
    referrerPolicy?: string;
  }

  interface DelHTMLAttributes<T> extends HTMLAttributes<T> {
    cite?: string;
    dateTime?: string;
  }

  interface InsHTMLAttributes<T> extends HTMLAttributes<T> {
    cite?: string;
    dateTime?: string;
  }

  interface ColHTMLAttributes<T> extends HTMLAttributes<T> {
    span?: number;
    width?: number | string;
  }

  interface ColgroupHTMLAttributes<T> extends HTMLAttributes<T> {
    span?: number;
  }

  interface TableHTMLAttributes<T> extends HTMLAttributes<T> {
    cellPadding?: number | string;
    cellSpacing?: number | string;
    summary?: string;
  }

  interface TdHTMLAttributes<T> extends HTMLAttributes<T> {
    colSpan?: number;
    headers?: string;
    rowSpan?: number;
    abbr?: string;
    scope?: string;
  }

  interface ThHTMLAttributes<T> extends HTMLAttributes<T> {
    abbr?: string;
    colSpan?: number;
    headers?: string;
    rowSpan?: number;
    scope?: string;
    sorted?: 'reversed' | 'ascending' | 'descending' | 'none';
  }

  interface ButtonHTMLAttributes<T> extends HTMLAttributes<T> {
    autoFocus?: boolean;
    disabled?: boolean;
    form?: string;
    formAction?: string;
    formEncType?: string;
    formMethod?: string;
    formNoValidate?: boolean;
    formTarget?: string;
    name?: string;
    type?: 'submit' | 'reset' | 'button';
    value?: string | string[] | number;
    onClick?: EventHandler<MouseEvent>;
  }

  interface FieldsetHTMLAttributes<T> extends HTMLAttributes<T> {
    disabled?: boolean;
    form?: string;
    name?: string;
  }

  interface FormHTMLAttributes<T> extends HTMLAttributes<T> {
    acceptCharset?: string;
    action?: string;
    autoComplete?: string;
    encType?: string;
    method?: string;
    name?: string;
    noValidate?: boolean;
    target?: string;
    onSubmit?: EventHandler<Event>;
  }

  interface InputHTMLAttributes<T> extends HTMLAttributes<T> {
    accept?: string;
    alt?: string;
    autoComplete?: string;
    autoFocus?: boolean;
    capture?: boolean | string;
    checked?: boolean;
    crossOrigin?: string;
    disabled?: boolean;
    form?: string;
    formAction?: string;
    formEncType?: string;
    formMethod?: string;
    formNoValidate?: boolean;
    formTarget?: string;
    height?: number | string;
    list?: string;
    max?: number | string;
    maxLength?: number;
    min?: number | string;
    minLength?: number;
    multiple?: boolean;
    name?: string;
    pattern?: string;
    placeholder?: string;
    readOnly?: boolean;
    required?: boolean;
    size?: number;
    src?: string;
    step?: number | string;
    type?: string;
    value?: string | number | readonly string[];
    width?: number | string;
    onChange?: EventHandler<Event>;
    onInput?: EventHandler<InputEvent>;
  }

  interface LabelHTMLAttributes<T> extends HTMLAttributes<T> {
    form?: string;
    htmlFor?: string;
  }

  interface MeterHTMLAttributes<T> extends HTMLAttributes<T> {
    form?: string;
    high?: number;
    low?: number;
    max?: number | string;
    min?: number | string;
    optimum?: number;
    value?: string | number | readonly string[];
  }

  interface OptgroupHTMLAttributes<T> extends HTMLAttributes<T> {
    disabled?: boolean;
    label?: string;
  }

  interface OptionHTMLAttributes<T> extends HTMLAttributes<T> {
    disabled?: boolean;
    label?: string;
    selected?: boolean;
    value?: string | number | readonly string[];
  }

  interface OutputHTMLAttributes<T> extends HTMLAttributes<T> {
    form?: string;
    htmlFor?: string;
    name?: string;
  }

  interface ProgressHTMLAttributes<T> extends HTMLAttributes<T> {
    max?: number | string;
    value?: string | number | readonly string[];
  }

  interface SelectHTMLAttributes<T> extends HTMLAttributes<T> {
    autoComplete?: string;
    autoFocus?: boolean;
    disabled?: boolean;
    form?: string;
    multiple?: boolean;
    name?: string;
    required?: boolean;
    size?: number;
    value?: string | number | readonly string[];
    onChange?: EventHandler<Event>;
  }

  interface TextareaHTMLAttributes<T> extends HTMLAttributes<T> {
    autoComplete?: string;
    autoFocus?: boolean;
    cols?: number;
    disabled?: boolean;
    form?: string;
    maxLength?: number;
    minLength?: number;
    name?: string;
    placeholder?: string;
    readOnly?: boolean;
    required?: boolean;
    rows?: number;
    value?: string | number | readonly string[];
    wrap?: string;
    onChange?: EventHandler<Event>;
    onInput?: EventHandler<InputEvent>;
  }

  interface DetailsHTMLAttributes<T> extends HTMLAttributes<T> {
    open?: boolean;
  }

  interface MenuHTMLAttributes<T> extends HTMLAttributes<T> {
    type?: string;
  }

  // ...можно добавить другие теги по необходимости

  // Для компонентов
  interface Element {}
  interface ElementClass {}
  interface IntrinsicAttributes {
    children?: unknown;
    [key: string]: unknown;
  }
}
