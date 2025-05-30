# Applicación Chat Backend

Proyecto planteado como aplicación web que permite la comunicación a tiempo real en base al protocolo de red WebSockets, este es el back-end, o sea, quien recibe todos los mensajes y se encarga de hacer broadcast a los distintos clientes

### Como Ejecutar:

El programa se puede correr en cualquier sistema operativo, sea Linux, Windows 11 o macOs. primero clonar el repositorio y luego seguir los siguientes pasos

1. Instalar un compilador de rust, en el caso del proyecto, utilizamos rustc para compilar

2. Correr el método `cargo build`

3. Correr el método `cargo run`

## Documentación

### Aclaraciones
El servidor solo permite recibir mensajes en cierto esquema establecido previamente, por lo tanto mensajes que no conformen este estilo serán ignorado, así prevenimos que se haga broadcast de información no necesaria o accidental

El archivo main.rs es compilado y luego corrido, a este se le cargan primero las dependencias externas (establecidas en el .toml) y luego las dependencias internas, o sea, modulos escritos por nosotros.

El esquema para establecer modulos es similar a c++, pero en este caso se usan archivos de la misma extension y no un .header, ya que todos los archivos son 'modulos' de por si (en rust se usa el termino crates), cada modulo tiene su "mod.rs" que es su header estableciendo un modulo que se exporta y luego sus distintos archivos, que el programa puede cargar posteriormente, cada modulo puede tener cuantos archivos desee.

### Sobre Rust

Err = catch
Ok = finally

Definimos estructuras de datos diciendo strut {} y construimos la estructura :v

### Librerias

Las librerias externas utilizadas fueron "tokio", "serde" y "chrono". 

Tokio permite la funcion asincronica de la función main y además permite conexiones Tcp, es mediante la cual creamos el listener (dentro del main.rs), y establecemos un puerto a abrir, es una libreria principalmente utilizada para conexiones TcP y uno de sus métodos incluye bindear un host, que usamos para abrir el servidor WebSocket

La libreria Serde la utilizamos para serializar y deserializar (ser-de) datos, es principalmente requerida en el método `user.send()` para serializar los datos a un Json y luego convertirlos en un mensaje de Tokio para enviar via websocket

Chrono es usada para enviar una timestamp del mensaje

### Estructura

El programa se inicia en main.rs, fuera de las carpetas de los modulos, este programa carga una conexión TCP abriendo un puerto específico usando la libreria tokio, usando el siguiente snippet:

```rs
let port = "0.0.0.0:8080" // escuchar cualquier puerto
let listener = TcpListener::bind(port).await?
```

Una vez establecido el puerto se espera a todos los headers que se quieran conectar al servidor, y cargamos la variable a un stream que luego extraemos con un método de tokio, todo el proceso de establecer y aceptar conexiones se realiza en la función `async fn handle_connection(tcp_stream)` del main, el sample de código siguiente extrae el "stream" de cada conexión

```rs
let ws_stream = tokio_tungstenite::accept_hdr_async(tcp_stream, callback).await?;
```

Para manejar de mejor manera los usuarios, hicimos una estructura de datos UserConnection, que posee una id y un transmitter (ya que su receptor no es necesario mas que cuando se carga el thread de manejar el receive de mensajes). Esta estructura se implementa en su propio control de clases y se separan el transmisor del receptor

```rs
impl UserConnection {
    pub async fn new(id: usize, stream: WebSocketStream<TcpStream>) {
        let (transmitter, receiver) = stream.split();

        return (
            UserConnection {
                id,
                transmitter,
            },

            receiver
        )
    }
}
```


### Futuras optimizaciones

Crear un modulo de mensajes para poder convertir mensajes a json y a un mensaje sin el uso de dos librerias, y solo llamar `user.send(to_message({}))` y así mejorar la lectura del código


