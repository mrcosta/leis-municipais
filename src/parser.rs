use crate::error::{CapturedOkOrUnexpected, Error};
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use html_sanitizer::TagParser;
use regex::Regex;
use serde::Serialize;
use std::fs::File;
use std::io::Read;

lazy_static! {
    static ref TITULO_REGEX: Regex = Regex::new("<h2>(?P<titulo>(.*))</h2>").unwrap();
    static ref RESUMO_REGEX: Regex = Regex::new("</h2><br>(?P<resumo>(.*))<br><br><img").unwrap();
    static ref TEXTO_REGEX: Regex = Regex::new("><br><br><br>(?P<texto>(.*))<p><img").unwrap();
    static ref DOCUMENTO_REGEX: Regex =
        Regex::new("btn-default\" href=\"(?P<documento>(.*))\" title").unwrap();
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Lei {
    titulo: String,
    categoria: String,
    resumo: String,
    texto: String,
    documento: Option<String>,
}

pub fn parse_html_to_lei(file_name: &str, categoria: String) -> Result<Lei, Error> {
    let file = File::open(file_name).expect("Arquivo que estava na pasta não foi encontrado");
    let mut transcoded = DecodeReaderBytesBuilder::new()
        .encoding(Some(WINDOWS_1252))
        .build(file);

    let mut dest = String::new();
    transcoded
        .read_to_string(&mut dest)
        .expect("O conteúdo do arquivo não é UTF-8 válido");

    let captures_titulo = TITULO_REGEX
        .captures(&dest)
        .ok_or_unexpected("Título", file_name)?;
    let captures_resumo = RESUMO_REGEX
        .captures(&dest)
        .ok_or_unexpected("Resumo", file_name)?;
    let captures_texto = TEXTO_REGEX
        .captures(&dest)
        .ok_or_unexpected("Texto", file_name)?;
    let documento = DOCUMENTO_REGEX
        .captures(&dest)
        .map(|captures_documento| captures_documento["documento"].to_string());

    Ok(Lei {
        titulo: clean_html_to_text(&captures_titulo["titulo"]),
        resumo: clean_html_to_text(&captures_resumo["resumo"]),
        texto: clean_html_to_text(&captures_texto["texto"]),
        documento,
        categoria,
    })
}

fn clean_html_to_text(capture: &str) -> String {
    let mut tag_parser = TagParser::new(&mut capture.as_bytes());
    tag_parser.walk(|tag| {
        if tag.name == "br" {
            tag.rewrite_as("\n".to_string());
        } else {
            tag.ignore_self();
        }
    })
}

#[cfg(test)]
mod test {
    use crate::parser::{parse_html_to_lei, Lei};

    #[test]
    fn should_read_html_and_create_a_lei_with_documento() {
        assert_eq!(
            parse_html_to_lei("resources/unit_tests/LeisMunicipais-com-br-Lei-Complementar-122-2019.html", "test".to_string()).unwrap(),
            Lei {
                titulo: "LEI COMPLEMENTAR Nº 122, DE 22 DE FEVEREIRO DE 2019".to_string(),
                resumo: "Altera as disposições da Lei Complementar Nº11/2002 que trata do modo de concessão de pensão por morte, em concordância a Lei Federal de nº 13.135 de 17/06/2015 e Nota Técnica nº 11/2015/CGNAL/DRPSP/SPPS, de 14/08/2015, e dá outras providências.".to_string(),
                texto: "O PREFEITO MUNICIPAL DE FEIRA DE SANTANA, Estado da Bahia, no uso de suas atribuições, FAÇO saber que a Câmara Municipal, através do Projeto de Lei Complementar Nº 12/2018, de autoria do Executivo, aprovou e eu sanciono a seguinte Lei:\n\nArt. 1ºFica alterado o artigo 48 da Lei Complementar nº11/2002, que passa viger com a seguinte redação:\n\n\"Art. 48. A pensão por morte será calculada na seguinte forma:\n\nI - ao valor da totalidade dos proventos do servidor falecido, até o limite máximo estabelecido para os benefícios do regime geral de previdência social de que trata o art. 201 da CF/88, acrescido de 70% (setenta por cento) da parcela excedente a este limite, caso aposentado na data do óbito; ou efetivo em que se deu o falecimento, até o limite máximo estabelecido para os benefícios do regime geral de previdência social de que trata o art. 201 da CF/88, acrescido de 70% (setenta por cento) da parcela excedente a este limite, caso em atividade na data do óbito.\n\n§ 1º A importância total assim obtida será rateada em partes iguais entre todos os dependentes com direito a pensão, e não será protelada pela falta de habilitação de outro possível dependente.\n\n§ 2º A habilitação posterior que importe inclusão ou exclusão de dependente só produzirá efeitos a contar da data da inscrição ou habilitação.\"\n\nArt. 2ºFica alterado o artigo 49 da Lei Complementar nº11/2002, que passa viger com a seguinte redação:\n\n\"Art. 49. Será concedida pensão provisória por morte presumida do segurado, nos seguintes casos: I - sentença declaratória de ausência, expedida por autoridade judiciária competente; e\n\nII - desaparecimento em acidente, desastre ou catástrofe devidamente evidenciados, desde que comprove que ingressou em Juízo para obter a competente sentença declaratória de ausência, caso em que a pensão provisória por morte presumida será devida até a prolação da sentença, momento a partir do qual o seu direito dependerá dos termos da decisão judicial.\n\n§ 1º A pensão provisória será transformada em definitiva com o óbito do segurado ausente ou deverá ser cancelada com o reaparecimento do mesmo, ficando os dependentes desobrigados da reposição dos valores recebidos, salvo comprovada má-fé.\n\n§ 2º Não fará jus a pensão o dependente condenado por prática de crime doloso de que tenha resultado a morte do segurado.\"\n\nArt. 3ºFica acrescido o artigo 50 à Lei Complementar nº11/2002, que passa a viger com a seguinte redação:\n\n\"Art. 50. A pensão por morte será devida ao conjunto dos dependentes do segurado que falecer, aposentado ou não, a contar da data:\n\nI - do óbito, quando requerida até trinta dias depois deste;\n\nII - do requerimento, quando requerida após o prazo previsto no inciso I; ou\n\nIII - da decisão judicial, no caso de morte presumida.\n\n§ 1º No caso do disposto no inciso II, não será devida qualquer importância relativa a período anterior à data de entrada do requerimento.\n\n§ 2º O direito a pensão configura-se na data do falecimento do segurado, sendo o benefício concedido com base na legislação vigente nessa data, vedado o recálculo em razão do reajustamento do limite máximo dos benefícios do RGPS.\"\n\nArt. 4ºFica alterado o artigo 51 da Lei Complementar nº11/2002, que passa a viger com a seguinte redação:\n\n\"Art. 51. A pensão por morte somente será devida ao filho e ao irmão inválido, cuja invalidez tenha ocorrido antes da emancipação ou de completar a maioridade civil, ressalvado o caso em que for comprovado pela perícia médica do IPFS a continuidade da invalidez, até a data do óbito do segurado.\n\n§ 1º A invalidez ou alteração de condições quanto ao dependente superveniente a morte do segurado, não dará origem a qualquer direito a pensão.\n\n§ 2º Os dependentes inválidos ficam obrigados, tanto para concessão como para manutenção e cessação de suas quotas de pensão, a submeterem-se aos exames médicos determinados pelo IPFS.\n\n§ 3º Ficam dispensados dos exames referidos neste artigo os pensionistas inválidos que atingirem a idade de 60 (sessenta) anos.\"\n\nArt. 5ºFica alterado o artigo 52 da Lei Complementar nº11/2002, que passa a viger com a seguinte redação:\n\n\"Art. 52. A pensão por morte, havendo mais de um pensionista, será rateada entre todos em parte iguais.\n\n§ 1º O direito a percepção de cada cota individual cessará:\n\nI - pela morte do pensionista;\n\nII - para filho, pessoa a ele equiparada ou irmão, de ambos os sexos, ao atingir a maioridade civil, salvo se for inválido ou com deficiência;\n\nIII - para filho ou irmão inválido, pela cessação da invalidez;\n\nIV - para filho ou irmão que tenha deficiência intelectual ou mental ou deficiência grave, pelo afastamento da deficiência, nos termos do regulamento;\n\nV - para cônjuge ou companheiro:\n\na) se inválido ou com deficiência, pela cessação da invalidez ou pelo afastamento da deficiência, respeitados os períodos mínimos decorrentes da aplicação das alíneas b e c;\nb) em 4 (quatro) meses, se o óbito ocorrer sem que o segurado tenha vertido 18 (dezoito) contribuições mensais ou se o casamento ou a união estável tiverem sido iniciados em menos de 2 (dois) anos antes do óbito do segurado;\nc) transcorridos os seguintes períodos, estabelecidos de acordo com a idade do beneficiário na data de óbito do segurado, se o óbito ocorrer depois de vertidas 18 (dezoito) contribuições mensais e pelo menos 2 (dois) anos após o início do casamento ou da união estável:\n\nI - 03 (três) anos, com menos de 21 (vinte e um) anos de idade;\n\nII - 06 (seis) anos, entre 21 (vinte e um) e 26 (vinte e seis) anos de idade;\n\nIII - 10 (dez) anos, entre 27 (vinte e sete) e 29 (vinte e nove) anos de idade;\n\nIV - 15 (quinze) anos, entre 30 (trinta) e 40 (quarenta) anos de idade;\n\nV - 20 (vinte) anos, entre 41 (quarenta e um) e 43 (quarenta e três) anos de idade;\n\nVI - 4443 Vitalícia, com 44 (quarenta e quatro) ou mais anos de idade.\n\n§ 2º Serão aplicados, conforme o caso, a regra contida na alínea a ou os prazos previstos na alínea c, ambas do inciso V do § 1º, se o óbito do segurado decorrer de acidente de qualquer natureza ou de doença profissional ou do trabalho, independentemente do recolhimento de 18 (dezoito) contribuições mensais ou da comprovação de 02 (dois) anos de casamento ou de união estável.\n\n§ 3º Após o transcurso de pelo menos 3 (três) anos e desde que nesse período se verifique o incremento mínimo de um ano inteiro na média nacional única, para ambos os sexos, correspondente à expectativa de sobrevida da população brasileira ao nascer, poderão ser fixadas, em números inteiros, novas idades para os fins previstos na alínea c do inciso V do § 1º, em ato do Ministro de Estado da Previdência Social, limitado o acréscimo na comparação com as idades anteriores ao referido incremento.\n\n§ 4º O tempo de contribuição ao Regime Próprio de Previdência Social (RPPS) ou ao Regime Geral de Previdência Social será considerado na contagem das 18 (dezoito) contribuições mensais de que tratam as alíneas b e c do inciso V do § 1º\"\n\nArt. 6ºFica alterado o artigo da Lei Complementar nº11/2002, que passa a viger com a seguinte redação:\n\n\"Art. 53. A critério da Administração, o beneficiário de pensão cuja preservação seja motivada por invalidez, por incapacidade ou por deficiência, poderá ser convocado a qualquer momento para avaliação das referidas condições.\"\n\nArt. 7ºFica alterado o artigo 54 da Lei Complementar nº11/2002, que passa a viger com a seguinte redação:\n\n\"Art. 54. Ressalvado o direito de opção, é vedada a percepção cumulativa de pensão, inclusive a deixada por mais de um cônjuge ou companheiro.\"\n\nArt. 8ºFica alterado o artigo 55 da Lei Complementar nº11/2002, que passa a viger com a seguinte redação:\n\n\"Art. 55. Toda vez que se extinguir uma parcela de pensão será procedido novo rateio da pensão em favor dos pensionistas remanescentes.\"\n\nArt. 9ºFica alterado o artigo 56 da Lei Complementar nº11/2002, que passa a viger com seguinte redação:\n\n\"Art. 56. Com a extinção da quota do último pensionista, extinta ficará também a pensão.\"\n\nArt. 10.Esta Lei Complementar entra em vigor na data de sua publicação, revogadas as disposições em contrário.\n\nGabinete do Prefeito, 22 de fevereiro de 2019\n\nCOLBERT MARTINS DA SILVA FILHO\nPREFEITO MUNICIPAL\n\nMARIO COSTA BORGES\nCHEFE DE GABINETE DO PREFEITO\n\nCLEUDSON SANTOS ALMEIDA\nPROCURADOR GERAL DO MUNICÍPIO\n\nANTÔNIO ALCIONE DA SILVA CEDRAZ DIRETOR PRESIDENTE DO INSTITUTO DE PREVIDÊNCIA DE FEIRA DE SANTANA PUBLICADO NO DIÁRIO OFICIAL ELETRÔNICO DIA 23 DE JANEIRO DE 2019.Download do documento".to_string(),
                documento: Some("https://leis.s3.amazonaws.com/originais/feira-de-santana-ba/2019/lc-122-2019-feira_de_santana-ba.doc".to_string()),
                categoria: "test".to_string(),
            }
        );
    }

    #[test]
    fn should_read_html_and_create_a_lei_without_documento() {
        assert_eq!(
            parse_html_to_lei(
                "resources/unit_tests/LeisMunicipais-com-br-Decreto-1-1984.html",
                "test".to_string()
            ).unwrap(),
            Lei {
                titulo: "DECRETO Nº 1/84, de 05 de janeiro de 1984".to_string(),
                resumo: "DISPÕE SOBRE O ENQUADRAMENTO DO FUNCIONALISMO DA CÂMARA MUNICIPAL DE FEIRA DE SANTANA, E DÁ OUTRAS PROVIDÊNCIAS.".to_string(),
                texto: "O PRESIDENTE DA CÂMARA MUNICIPAL DE FEIRA DE SANTANA, estado da Bahia,no uso de suas atribuições conferidas pelo do art..32, XX, do Regimento Interno, e cumprimento determinações constantes do artigo 20, da lei municipal nº935/83, decreta:\n\nArt. 1ºFica aprovada a lista de enquadramento e classificação dos funcionários Câmara municipal de Feira de Santana efetivos e efetivados na data de aprovação da Lei Municipal nº935/ 53, constante do Anexo I.\n\nArt. 2ºOs titulares dos Cargos isolados de Provimento Efetivo e os Provimentos em Comissão já enquadrados na própria Lei935/83 continuarão a exercer as suas funções segundo o organograma Anexo IV da mesma Lei.\n\nArt. 3ºEste Decreto entrará em vigor na data de sua publicação e seus efeitos a partir de 1º de janeiro de 1984.\n\nGabinete da Presidência da Câmara Município de Feira de Santana.\n\nDIVAL FIGUEIREDO MACHADO\nPresidente\n\nLISTA DE CLASSIFICAÇÃO DOS FUNCIONÁRIOS de acordo com a lei Municipal nº935de 02/12/83__________________________________________________________________________________\n|Nº DE|     NOME DO FUNCIONÁRIO     |CARGO ANTERIOR| CARGO ATUAL SÍMB. |NOVO GRUPO |\n|ORDEM|                             |              |                   |OCUPACIONAL|\n|=====|=============================|==============|===================|===========|\n|  01 |Charles Marques de Sant´Ana. | Mensag.      |Aux.Ser.Ge.  SG-1  |Set.Admin. |\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  02 |Mª. De Lourdes Ferreira Alves| Servente     |Aux.Ser.Ge.  SG-1  |Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  03 |Izaltina Santos              | Servente     |Aux.Ser.Ge.  SG-1  |Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  04 |Vilma Ferreira da Silva      | Servente     |Aux.Ser.Ge.  SG-1  |Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  05 |Valmir Alves de Sena         | Vigilante    |Aux.Ser.Ge.  SG-2  |Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  06 |Olimpio Pereira da Silva     | Vigilante    |Aux.Ser.Ge.  SG-2  |Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  07 |Lourival F. do Nascimento    | Vigilante    |Aux.Ser.Ge.  SG-2  |Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  08 |Claudemiro da Silva Oliveira | Vigilante    |Aux.Ser.Ge.  SG-2  |Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  09 |Joselito Carvalho Venas.     | Vigilante    |Aux.Ser.Ge.  SG-2  |Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  10 |Elias de Azevedo.            | Vigilante    |Aux.Ser.Ge.  SG-2  |Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  11 |Júlio Soares de Souza.       | Op. Grav.    |Aux.Ser.Ge.  SG-2  |Set.Legisl.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  12 |Pelúcio Rodrigues Filho      | Mensag.      |Aux.Ser.Ge.  SG-5  |Set.Legisl.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  13 |Paulino Gonçalves da Silva   | Almoxarifado |Aux.Ser.Ge.  SG-5  |Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  14 |Tertuliano dos Santos Reis.  | Porteiro     |Aux.Ser.Ge.  SG-5  |Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  15 |Elisiana Alves Santana       | Telefonista  |Aux.Lesgisl. AL - 1|Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  16 |Anisía Maria da Silva        | Recepcionista|Aux.Lesgisl. AL - 1|Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  17 |Valderez Santos Bispo        | Datilog.     |Aux.Lesgisl. AL - 1|Set.Financ.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  18 |Mª. Cristina Alves da Silva. | Datilog.     |Aux.Lesgisl. AL - 1|Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  19 |Uilma Moreira Silva.         | Datilog.     |Aux.Lesgisl. AL - 2|Set.Legisl.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  20 |Edson de Oliveira Matos      | Mensag.      |Aux.Lesgisl. AL - 2|Set.Financ.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  21 |Marcos Antônio da Silva      | Mensag.      |Aux.Lesgisl. AL - 3|Set.Legisl.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  22 |Doranei Cedraz V. da Silveira| Datilog.     |Aux.Lesgisl. AL - 3|Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  23 |Mª. das Dores Falcão Pedreira| Arquivo.     |Aux.Lesgisl. AL - 3|Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  24 |Mª. Zenilda de Souza Lima    | Datilog.     |Aux.Lesgisl. AL - 4|Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  25 |Leda Lima de Azevedo         | Datilog.     |Aux.Lesgisl. AL - 5|Set.Financ.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  26 |Eunira Pinheiro Xavier       | Aux.Adm.     |Aux.Lesgisl. AL - 6|Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  27 |Éclair Cedraz de Oliveira    | Aux. Tes.    |Aux.Lesgisl. AL - 7|Set.Legisl.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  28 |Angélica Mª. Daltro Lopes.   | Red. Deb.    |Aux.Lesgisl. AL - 8|Set.Legisl.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  29 |Nílton de Oliveira Caribé.   | Red. Deb.    |Ofic. egisl. OL - 1|Set.Legisl.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  30 |Rossini Souza                | Red. Deb.    |Ofic.Legisl. OL - 2|Set.Legisl.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  31 |Edivaldo de Jesus Xavier     | Aux. Cont.   |Tec. Contab. TC - 1|Set.Financ.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  32 |Erideth Santos Lopes         | Tesour.      |Tec. Contab. TC - 2|Set.Financ.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  33 |Edeltrudes Sousa Costa       | Contador     |Tec. Contab. TC - 5|Set.Financ.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  34 |Manoel Ernesto da Costa      | Motorist.    |Motorista    MP - 1|Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  35 |Fernando A. Brito Valadão    | Motorist.    |Motorista    MP - 1|Set. Admin.|\n|-----|-----------------------------|--------------|-------------------|-----------|\n|  36 |Renildo Domingos dos Santos. | Motorist.    |Motorista    MP - 2|Set. Admin.|\n|_____|_____________________________|______________|___________________|___________| * tabela formatada pela equipe técnica do LeisMunicipais.com.br\nGabinete da Presidência da Câmara Município de Feira de Santana, 05 de Janeiro de 1984.\n\nDIVAL FIGUEIREDO MACHADO\nPresidente".to_string(),
                documento: None,
                categoria: "test".to_string(),
            }
        );
    }

    #[test]
    fn should_return_pattern_not_found_error_when_titulo_pattern_not_found() {
        let result = parse_html_to_lei(
            "resources/unit_tests/Leis_sem_titulo_comh2.html",
            "test".to_string(),
        );

        assert_eq!(
            &format!("{}", &result.unwrap_err()),
            "Título não encontrado no arquivo resources/unit_tests/Leis_sem_titulo_comh2.html"
        );
    }

    #[test]
    fn should_return_pattern_not_found_error_when_resumo_pattern_not_found() {
        let result = parse_html_to_lei(
            "resources/unit_tests/Leis_sem_resumo.html",
            "test".to_string(),
        );

        assert_eq!(
            &format!("{}", &result.unwrap_err()),
            "Resumo não encontrado no arquivo resources/unit_tests/Leis_sem_resumo.html"
        );
    }

    #[test]
    fn should_return_pattern_not_found_error_when_texto_pattern_not_found() {
        let result = parse_html_to_lei(
            "resources/unit_tests/Leis_sem_texto.html",
            "test".to_string(),
        );

        assert_eq!(
            &format!("{}", &result.unwrap_err()),
            "Texto não encontrado no arquivo resources/unit_tests/Leis_sem_texto.html"
        );
    }

    // fn should_read_html_and_create_a_lei_from_it_without_download_documento_in_texto_property() {
}
