# Exemplo Prático: Prevenção de Divagações com Parâmetro "stop"
## Cenário de Uso

Um assistente virtual de um banco precisa responder sobre taxas de juros de forma concisamente objetiva, sem entrar em explicações extensas, opiniões ou comparações com outros bancos.

## Prompt Inicial (Sem Controle)

```text
Usuário: Qual a taxa do cheque especial do seu banco?

Assistente: A taxa do cheque especial no nosso banco atualmente é de 8,9% ao mês. É importante mencionar que essa taxa pode variar conforme o perfil do cliente e a análise de crédito. Comparando com o mercado, essa é uma taxa competitiva, já que a média do setor gira em torno de 9,5%. Aliás, falando em taxas, você sabia que muitos bancos cobram taxas ocultas? O nosso banco tem total transparência. Posso explicar como funciona o cálculo dos juros compostos? Basicamente...
```

### Problema: O modelo divagou para:

    Comparações com mercado

    Informações não solicitadas sobre transparência

    Oferecimento não solicitado de explicações adicionais

    Início de explicação sobre juros compostos

## Implementação com Parâmetro "stop"
### Versão 1: Controle Básico
```python
import openai

resposta = openai.ChatCompletion.create(
    model="gpt-3.5-turbo",
    messages=[
        {"role": "system", "content": "Seja conciso. Responda APENAS ao que foi perguntado."},
        {"role": "user", "content": "Qual a taxa do cheque especial do seu banco?"}
    ],
    stop=["\n\n", "Além disso", "Comparando", "Você sabia", "Posso", "Basicamente"]
)
```
### Saída possível:
```text
A taxa do cheque especial no nosso banco é de 8,9% ao mês.
```
(Para ao tentar adicionar "\n\n" para nova explicação)

## Versão 2: Controle Mais Refinado
```python
resposta = openai.ChatCompletion.create(
    model="gpt-4",
    messages=[
        {"role": "system", "content": """
        Instruções estritas:
        1. Responda APENAS com a informação solicitada
        2. Não faça comparações
        3. Não ofereça informações adicionais
        4. Não explique conceitos
        5. Formato máximo: 1-2 frases
        """},
        {"role": "user", "content": "Qual a taxa do cheque especial do seu banco?"}
    ],
    temperature=0.3,
    max_tokens=100,
    stop=[
        "\n\n",  # Impede novos parágrafos
        " Além",  # Impede "Além disso/disso"
        " Comparando",  # Impede comparações
        " Vale ressaltar",  # Impede ênfases desnecessárias
        " É importante",  # Impede explicações adicionais
        " Posso",  # Impede ofertas de ajuda
        " Basicamente",  # Impede simplificações
        " Em resumo",  # Impede resumos não solicitados
        " Aproveito"  # Impede cross-selling
    ]
)
```

## Comparação de Resultados
### Sem "stop" (comum):
```text
A taxa do cheque especial é de 8,9% ao mês. Esta taxa é definida com base no seu perfil de cliente e pode ser renegociada. Comparando com outros bancos, estamos 0,6% abaixo da média do mercado. Além disso, temos opções de crédito pessoal a partir de 4,9% ao mês, caso tenha interesse. Posso explicar como funcionam os juros compostos?
```
### Com "stop" eficaz:
```text
A taxa atual do cheque especial é de 8,9% ao mês.
```
ou 
```text
Nossa taxa de cheque especial é 8,9% ao mês, variável conforme análise de crédito.
```

## Análise Crítica do Exemplo

### Pontos Fortes Demonstrados:
    - Corta divagações no início: Impede que o modelo desenvolva tangentes

    - Mantém foco na pergunta: Resposta direta ao que foi perguntado

    - Evita cross-selling não solicitado: Impede oferta de outros produtos

    - Controla comprimento: Limita resposta a informações essenciais

### Problemas Práticos Encontrados:
#### Falsos positivos:
```python
# Se o modelo gerar:
# "A taxa é 8,9% ao mês. Além disso..." → PARADA CORRETA
# "A taxa é 8,9% ao mês. Além de competitiva..." → PARADA PREMATURA
# A palavra "Além" pararia mesmo quando faz parte de construção válida
```
#### Evasão criativa do modelo:
```python
# O modelo pode aprender a contornar:
stop = ["Comparando", "Além disso"]

# Resposta do modelo:
# "A taxa é 8,9%. Em relação ao mercado, é competitiva."
# "A taxa é 8,9%. Adicionalmente, varia por perfil."
# As palavras "Em relação" e "Adicionalmente" não estão no stop
```
#### Complexidade crescente:
```python
# Para ser eficaz, precisa-se de lista extensa:
stop = [
    "\n\n", "Além", "Comparando", "Também", "Adicionalmente",
    "Outro ponto", "Vale mencionar", "É interessante",
    "Aproveito", "Sabia que", "Posso", "Basicamente",
    "Em outras palavras", "Ou seja", "Isto é"
]
```
## Implementação Híbrida (Mais Robusta)
```python
def resposta_controlada(prompt_usuario, max_frases=2):
    """
    Sistema híbrido: stop + pós-processamento
    """
    # 1. Geração com parâmetro stop
    resposta_bruta = gerar_resposta(
        prompt_usuario,
        stop=["\n\n", "###", "---"],  # Stop simples
        temperature=0.3,
        max_tokens=200
    )
    
    # 2. Pós-processamento para prevenir divagações
    resposta_filtrada = filtrar_divagacoes(resposta_bruta, max_frases)
    
    return resposta_filtrada

def filtrar_divagacoes(texto, max_frases):
    """
    Remove divagações baseado em heurísticas
    """
    frases = texto.split('. ')
    
    if len(frases) <= max_frases:
        return texto
    
    # Identifica início de divagações
    marcadores_divagacao = [
        'além disso', 'comparando', 'outro ponto',
        'também é importante', 'vale destacar'
    ]
    
    for i, frase in enumerate(frases):
        frase_lower = frase.lower()
        if any(marcador in frase_lower for marcador in marcadores_divagacao):
            if i >= max_frases:  # Já passou do limite
                return '. '.join(frases[:max_frases]) + '.'
    
    return '. '.join(frases[:max_frases]) + '.'
```

## Cenário Real com Limitações
### Prompt do usuário:
```text
"O que é inflação?"
```
### Resposta sem controle (GPT-3.5, temperature=0.7):
```text
Inflação é o aumento geral dos preços em uma economia. Isso ocorre quando há mais dinheiro circulando do que bens e serviços disponíveis. Existem vários tipos de inflação: demanda, custos e inercial. No Brasil, o IPCA é o índice oficial. É importante investir para proteger seu dinheiro da inflação. Posso sugerir alguns investimentos indexados ao IPCA?
```
### Resposta com stop=["\n\n", "Posso", "É importante"]:
```
Inflação é o aumento geral dos preços em uma economia. Isso ocorre quando há mais dinheiro circulando do que bens e serviços disponíveis. Existem vários tipos de inflação: demanda, custos e inercial. No Brasil, o IPCA é o índice oficial. É importante investir para proteger seu dinheiro
```
(Cortou no meio da frase, removendo contexto útil)

## Conclusão Prática
O parâmetro "stop" ajuda na prevenção de divagações, mas:

    1. É uma linha de defesa primária, não completa

    2. Funciona melhor combinado com:

        max_tokens baixo

        temperature baixa

        Prompt engineering preciso

        Pós-processamento

    3. Para casos críticos, técnicas superiores incluem:

        Fine-tuning com respostas concisas

        Retrieval Augmented Generation (RAG) para manter contexto

        Sistemas de classificação de relevância

        Gramáticas controladas (como Guidance ou Outlines)

O exemplo mostra que enquanto "stop" pode prevenir algumas divagações, ele é uma ferramenta grosseira que frequentemente corta tanto informação relevante quanto irrelevante, exigindo ajuste fino constante conforme o modelo e o contexto mudam.
