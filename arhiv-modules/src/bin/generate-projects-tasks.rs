use arhiv::Arhiv;
use arhiv_modules::generator::*;
use arhiv_modules::*;
use rand::prelude::*;
use rand::thread_rng;

static COMPLEXITY: &'static [TaskComplexity] = &[
    TaskComplexity::Unknown,
    TaskComplexity::Small,
    TaskComplexity::Medium,
    TaskComplexity::Large,
    TaskComplexity::Epic,
];

static STATUS: &'static [TaskStatus] = &[
    TaskStatus::Inbox,
    TaskStatus::Todo,
    TaskStatus::Later,
    TaskStatus::InProgress,
    TaskStatus::Paused,
    TaskStatus::Done,
    TaskStatus::Cancelled,
];

#[tokio::main]
async fn main() {
    env_logger::init();

    let arhiv = Arhiv::must_open();
    let attachments = create_attachments();
    let generator = Generator::new(&attachments);

    for _ in 0..15 {
        // generate 15 projects
        let mut project = Project::new();

        project.0.data = ProjectData {
            title: generator.gen_string(),
            description: generator.gen_markup_string(1, 2),
        };

        let project_id = project.0.id.clone();

        arhiv
            .stage_document(project.into_document(), attachments.clone())
            .expect("must be able to save document");

        let mut rng = thread_rng();

        for _ in 0..rng.gen_range(5, 30) {
            let project_id = project_id.clone();

            let mut task = Task::new(project_id.clone());

            task.0.data = TaskData {
                project_id,
                title: generator.gen_string(),
                description: generator.gen_markup_string(0, 1),
                complexity: *COMPLEXITY.choose(&mut rng).unwrap(),
                status: *STATUS.choose(&mut rng).unwrap(),
            };

            arhiv
                .stage_document(task.into_document(), attachments.clone())
                .expect("must be able to save document");
        }
    }

    arhiv.sync().await.expect("must be able to sync");
}
