export interface GradioEndpoint {
    name: string
    index: number
    parameters: GradioParameter[]
    returns: GradioReturns[]
}
export interface GradioType {
    type: string
}

export interface PythonType {
    type: string
    description: string
}

export interface GradioParameter {
    label: string
    parameter_name: string
    parameter_has_default: boolean
    parameter_default: string
    type: GradioType
    python_type: PythonType
    component: string
    example_input: string
}

export interface Path {
    title: string
    type: string
}

export interface PropertyMember {
    anyOf: GradioType[]
    default?: any
    title: string
}

export interface Is_stream {
    default: boolean
    title: string
    type: string
}

export interface Default {
    _type: string
}

export interface Meta {
    default: Default
    title: string
    type: string
}

export interface Property {
    path: Path
    url: PropertyMember
    size: PropertyMember
    orig_name: PropertyMember
    mime_type: PropertyMember
    is_stream: Is_stream
    meta: Meta
}

export interface GradioReturnType {
    properties: Property
    required: string[]
    title: string
    type: string
}

export interface GradioReturns {
    label: string
    type: GradioReturnType
    python_type: PythonType
    component: string
}

export interface PropertyEntry {
    path: string
    url: string
    size: number
    orig_name: string
    mime_type: string
    is_stream: boolean
    meta: any
}

export class GradioClient {
    session_hash: string
    named_endpoints: Record<string, GradioEndpoint> = {}
    constructor(public url: string) {
        this.session_hash = Math.random().toString(16).slice(2, 12)
    }
    static async connect(url: string) {
        if (url.endsWith("/")) url = url.slice(0, -1)
        const client = new GradioClient(url)
        await client.init()
        return client
    }
    async api(endpoint: string) {
        const res = await fetch(`${this.url}${endpoint}`, {
            headers: {
                "Content-Type": "application/json",
            },
        })
        return res.json()
    }
    async raw_call<T = string>(endpoint: string, data: any) {
        const res = await fetch(`${this.url}/call${endpoint}`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ data }),
        })
        const { event_id } = await res.json()
        if (!event_id) Promise.reject("err")
        return new Promise<T>((resolve) => {
            const ev = new EventSource(`${this.url}/call${endpoint}/${event_id}`)
            ev.addEventListener("complete", (e) => {
                ev.close()
                const data = JSON.parse(e.data) as PropertyEntry[]
                if (data.length) {
                    if (data[0].path) {
                        resolve(this.file(data[0].path) as any)
                    } else {
                        resolve(data[0] as any)
                    }
                }
            })
        })
    }
    async raw_join(fn_index: number, data: any) {
        const res = await fetch(`${this.url}/queue/join`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({ data, event_data: null, fn_index, session_hash: this.session_hash }),
        })
        const { event_id } = await res.json()
        if (!event_id) Promise.reject("0")
        return new Promise((resolve, reject) => {
            const ev = new EventSource(`${this.url}/queue/data?session_hash=${this.session_hash}`)
            ev.onmessage = (e) => {
                const data = JSON.parse(e.data)
                console.debug(data)
                if (data.msg === "process_completed") {
                    resolve(data.output)
                } else if (data.msg === "close_stream") {
                    ev.close()
                    reject("error")
                }
            }
        })
    }
    async upload(blob: Blob) {
        const formData = new FormData()
        formData.append("files", blob)
        const res = await fetch(`${this.url}/upload?upload_id=${this.session_hash}`, {
            method: "POST",
            body: formData,
        })
        const [path] = await res.json()
        if (!path) throw new Error("error")
        return path
    }
    async init() {
        const res = await this.api("/info")
        if (res.named_endpoints) {
            this.named_endpoints = Object.fromEntries(
                Object.entries(res.named_endpoints).map(
                    ([name, endpoint]: [string, any], index) =>
                        [name, { name, index, parameters: endpoint.parameters, returns: endpoint.returns }] as [string, GradioEndpoint]
                )
            )
        }
    }
    async call<T = string>(endpoint: string, data: any) {
        const endpoint_info = this.named_endpoints[endpoint]
        if (!endpoint_info) {
            throw new Error(`Endpoint ${endpoint} not found`)
        }
        const mapped_data = this.mapParameters(endpoint, data)
        return await this.raw_call<T>(endpoint, mapped_data)
    }
    mapParameters(endpoint: string, data: any) {
        const endpoint_info = this.named_endpoints[endpoint]
        if (!endpoint_info) {
            throw new Error(`Endpoint ${endpoint} not found`)
        }
        const mapped_data = endpoint_info.parameters.map((parameter) => {
            const { parameter_name, parameter_has_default, parameter_default } = parameter
            if (parameter_name in data) {
                return data[parameter_name]
            } else if (parameter_has_default || parameter_default == null) {
                return parameter_default
            } else {
                throw new Error(`Missing required parameter ${parameter_name} for endpoint ${endpoint}`)
            }
        })
        return mapped_data
    }
    file(name: string) {
        return `${this.url}/file=${name}`
    }
}
