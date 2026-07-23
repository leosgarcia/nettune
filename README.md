# NetTune 🏎️

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)
![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange)
![Tauri](https://img.shields.io/badge/Tauri-2.0-yellow)

NetTune é uma ferramenta desktop multiplataforma construída em **Rust** e **Tauri**, focada em aplicar ajustes avançados nas configurações de rede nativas do seu Sistema Operacional. Seu objetivo principal é **reduzir o *jitter* e estabilizar a latência**, o que beneficia enormemente aplicações interativas e jogos competitivos (esports).

⚠️ **O QUE O NETTUNE NÃO É:** 
O NetTune **não** é uma VPN, um Proxy ou um serviço de Tunelamento. Ele **NÃO** intercepta, redireciona ou injeta pacotes na sua conexão. Todas as otimizações são feitas configurando parâmetros locais já existentes no kernel do seu Sistema Operacional (via *Registry* no Windows ou *sysctl* no UNIX).

---

## 🎯 Funcionalidades e Ajustes Suportados

A ferramenta abstrai a complexidade dos ajustes de rede através de uma interface intuitiva, garantindo total transparência e segurança através de um **Modo Dry-Run** e sistema inteligente de **Rollback automático**.

### Windows
- **Nagle's Algorithm (`TcpAckFrequency` & `TCPNoDelay`)**: Desabilita o atraso de reconhecimento de pacotes TCP por placa de rede ativa.
- **Network Throttling Index**: Desativa a limitação do Windows para tráfego multimídia, liberando recursos máximos de CPU para processamento de pacotes de jogos.
- **Interrupt Moderation**: Desabilita o atraso proposital de processamento na placa de rede, trocando uso de CPU por menor latência absoluta na entrega.

### Linux
- **TCP BBR (Congestion Control)**: Ativa o algoritmo de controle de congestionamento moderno da Google.
- **Netdev Max Backlog**: Aumenta os buffers internos do kernel para tolerar picos intensos (bursts) sem perder pacotes.
- **TCP Low Latency**: Força a pilha de rede a processar pacotes individualmente sem acumular (batching).
- **Interrupt Coalescing**: Remove o delay em hardware (usando `ethtool`) focado em performance pura.

### macOS
- **TCP Delayed ACK**: Responde imediatamente a pacotes curtos, acelerando tráfego responsivo.
- **TCP Window Size**: Ajusta buffers `sendspace` e `recvspace` da pilha. (O impacto visual no macOS moderno pode ser marginal).

---

## 🛡️ Segurança, Backups e Riscos

**Riscos Envolvidos:** Mexer com TCP pode aumentar o consumo de CPU da sua máquina ou, em redes Wi-Fi ruins, aumentar a degradação de banda bruta (throughput) em favor da velocidade (latência). 

**Proteção contra falhas:** 
Sempre que um ajuste é aplicado, o NetTune lê o valor *original* direto do seu sistema operacional (seja o DWORD original do Windows ou a flag no Linux) e salva em `~/.nettune/backup.json`. Você sempre pode usar o comando **"Restaurar Tudo"**.

**Recuperação Manual:**
Caso você perca o acesso ao aplicativo e algo saia errado, basta deletar as chaves no Editor de Registro (`regedit`) correspondentes listadas acima no Windows, ou reiniciar o computador em sistemas UNIX (a maioria das mudanças em sysctl/ethtool são limpas após o reboot caso não tenham sido gravadas persistentemente na rc).

---

## 💻 Como Usar

O NetTune foi construído modularmente, fornecendo tanto uma **Interface Gráfica (GUI)** lindíssima focada no usuário, quanto uma **Interface de Linha de Comando (CLI)** para automação e *Power Users*.

### Opção 1: Usando a Interface Gráfica (Tauri)
Requer `npm` e dependências de desenvolvimento do Tauri.

```bash
cd gui
npm install
npm run tauri dev
```
A interface gráfica é equipada com um modo de Simulação (Dry-Run). Ative-o na barra de ferramentas superior para ver o que seria alterado sem medo de estragar sua conexão.

### Opção 2: Usando a CLI (Rust)

Acesse o diretório raiz e rode a interface de comando. No Windows, abra o PowerShell **Como Administrador** se for aplicar algo. No Linux/Mac, use `sudo` ao invocar o executável final.

```bash
# Listar o status do seu sistema operacional
cargo run --bin cli -- status

# Listar quais tweaks o seu SO suporta nativamente
cargo run --bin cli -- list

# Aplicar todos os ajustes
cargo run --bin cli -- apply-all

# Testar o modo Dry-Run (não faz nada de verdade)
cargo run --bin cli -- apply-all --dry-run

# Reverter um ajuste específico de volta ao padrão do Windows
cargo run --bin cli -- revert win_network_throttling

# Reverter todos os ajustes e restaurar ao sistema de fábrica
cargo run --bin cli -- revert-all
```

---

## 🏗️ Arquitetura

1. `core/`: Motor em Rust responsável por interagir diretamente com APIs de sistema de cada SO (Registry/PowerShell/Sysctl).
2. `cli/`: Um wrapper `clap` e `tracing` super leve que consome o `core`.
3. `gui/`: Um app Tauri moderno construído com **Svelte** e Vanilla CSS (Glassmorphism), servindo como camada frontend que também consome diretamente os métodos do `core`.

## 📜 Licença e Privacidade

O código é livre e aberto sob a licença **MIT**. Nenhuma métrica, telemetria analítica, ou endereço IP é coletado e enviado remotamente. A ferramenta opera localmente sem necessidade de conexão externa ativa.

---

<p align="center">
  <a href="https://buymeacoffee.com/leosgarcia" target="_blank">
    <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" style="height: 60px !important; width: 217px !important;">
  </a>
</p>
