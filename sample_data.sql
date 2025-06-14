-- Sample data to test the server

-- Development projects
INSERT INTO Dev_Project_Metadata (slug, en_title, en_short_description, fr_title, fr_short_description, techs, link, date, tags, priority) VALUES 
('portfolio-server', 'Portfolio Server', 'A lightweight Rust server for portfolio content', 'Serveur Portfolio', 'Un serveur Rust léger pour le contenu de portfolio', 'Rust,Axum,PostgreSQL', 'https://github.com/username/portfolio-server', '2025-06-13', 'web,backend,api', 1),
('photo-gallery', 'Photo Gallery App', 'Modern photo gallery with responsive design', 'Application Galerie Photo', 'Galerie photo moderne avec design responsive', 'React,TypeScript,Tailwind', 'https://github.com/username/photo-gallery', '2025-05-20', 'frontend,react,photography', 2);

-- Photo albums
INSERT INTO Album_Metadata (slug, title, description, short_title, date, camera, lens, phone, preview_img_one_url, featured, category) VALUES 
('urban-exploration', 'Urban Exploration 2025', 'Exploring the city through photography', 'Urban 2025', '2025-06-01', 'Canon EOS R5', 'RF 24-70mm f/2.8L', NULL, '/files/urban-exploration/preview1.jpg', true, 'Street'),
('nature-walks', 'Nature Walks', 'Peaceful moments in nature', 'Nature', '2025-05-15', NULL, NULL, 'iPhone 15 Pro', '/files/nature-walks/preview1.jpg', false, 'Nature');

-- Album content (adapt according to your actual images)
INSERT INTO Album_Content (slug, img_url, caption) VALUES 
('urban-exploration', '/files/urban-exploration/street1.jpg', 'Street art in downtown'),
('urban-exploration', '/files/urban-exploration/building1.jpg', 'Modern architecture'),
('nature-walks', '/files/nature-walks/forest1.jpg', 'Morning light through trees');
